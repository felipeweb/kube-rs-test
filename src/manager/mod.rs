pub mod metrics;
pub mod telemetry;
use crate::{ Error, Result};
use chrono::prelude::*;
use futures::{future::BoxFuture, FutureExt, StreamExt};
use kube::{
    api::{Api, ListParams, Patch, PatchParams, ResourceExt},
    client::Client,
    CustomResource, Resource,
};
use prometheus::{
    default_registry, proto::MetricFamily,
};
use kube_runtime::controller::{Context, Controller, ReconcilerAction};
use maplit::hashmap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::RwLock,
    time::{Duration, Instant},
};
use tracing::{debug, error, event, field, info, instrument, trace, warn, Level, Span};

/// Our Foo custom resource spec
#[derive(CustomResource, Deserialize, Serialize, Clone, PartialEq, Debug, JsonSchema)]
#[kube(
    kind = "Foo",
    group = "clux.dev",
    version = "v1",
    derive = "PartialEq",
    struct = "Foo",
    status = "FooStatus",
    namespaced
)]
pub struct FooSpec {
    name: String,
    info: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug, JsonSchema)]
pub struct FooStatus {
    is_bad: bool,
    last_updated: Option<DateTime<Utc>>,
}

// Context for our reconciler
#[derive(Clone)]
struct Data {
    /// kubernetes client
    client: Client,

    /// Various prometheus metrics
    metrics: metrics::Metrics,
}

#[instrument(skip(ctx), fields(trace_id))]
async fn reconcile(foo: Foo, ctx: Context<Data>) -> Result<ReconcilerAction, Error> {
    let trace_id = telemetry::get_trace_id();
    Span::current().record("trace_id", &field::display(&trace_id));
    let start = Instant::now();

    let client = ctx.get_ref().client.clone();
    let name = ResourceExt::name(&foo);
    let ns = ResourceExt::namespace(&foo).expect("foo is namespaced");
    let foos: Api<Foo> = Api::namespaced(client, &ns);

    let new_status = Patch::Apply(json!({
        "apiVersion": "clux.dev/v1",
        "kind": "Foo",
        "status": FooStatus {
            is_bad: foo.spec.info.contains("bad"),
            last_updated: Some(Utc::now()),
        }
    }));
    let ps = PatchParams::apply("cntrlr").force();
    let _o = foos
        .patch_status(&name, &ps, &new_status)
        .await
        .map_err(Error::KubeError)?;

    let duration = start.elapsed().as_millis() as f64 / 1000.0;
    ctx.get_ref()
        .metrics
        .reconcile_duration
        .with_label_values(&[])
        .observe(duration);
    ctx.get_ref().metrics.handled_events.inc();
    info!("Reconciled Foo \"{}\" in {}", name, ns);

    // If no events were received, check back every 30 minutes
    Ok(ReconcilerAction {
        requeue_after: Some(Duration::from_secs(3600 / 2)),
    })
}

fn error_policy(error: &Error, _ctx: Context<Data>) -> ReconcilerAction {
    warn!("reconcile failed: {:?}", error);
    ReconcilerAction {
        requeue_after: Some(Duration::from_secs(360)),
    }
}

/// Data owned by the Manager
#[derive(Clone)]
pub struct Manager {
    /// Various prometheus metrics
    metrics: metrics::Metrics,
}

/// Example Manager that owns a Controller for Foo
impl Manager {
    /// Lifecycle initialization interface for app
    ///
    /// This returns a `Manager` that drives a `Controller` + a future to be awaited
    /// It is up to `main` to wait for the controller stream.
    pub async fn new() -> (Self, BoxFuture<'static, ()>) {
        let client = Client::try_default().await.expect("create client");
        let metrics = metrics::Metrics::new();
        let context = Context::new(Data {
            client: client.clone(),
            metrics: metrics.clone(),
        });

        let foos = Api::<Foo>::all(client);
        // Ensure CRD is installed before loop-watching
        let _r = foos.list(&ListParams::default().limit(1)).await.expect(
            "is the crd installed? please run: cargo run --bin crdgen | kubectl apply -f -",
        );

        // All good. Start controller and return its future.
        let drainer = Controller::new(foos, ListParams::default())
            .run(reconcile, error_policy, context)
            .filter_map(|x| async move { std::result::Result::ok(x) })
            .for_each(|_| futures::future::ready(()))
            .boxed();

        (Self { metrics }, drainer)
    }

    /// Metrics getter
    pub fn metrics(&self) -> Vec<MetricFamily> {
        default_registry().gather()
    }
}
