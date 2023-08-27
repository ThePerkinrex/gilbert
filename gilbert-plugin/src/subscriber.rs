use std::{collections::HashMap, marker::PhantomData};

use gilbert_plugin_api::log::LogMessage;
use tracing::{field::Visit, Subscriber};
use tracing_subscriber::Layer;

use crate::sender::Sender;

pub struct LoggingLayer<S: Sender<T>, T: From<LogMessage>> {
    sender: S,
    _ph: PhantomData<T>,
}

impl<S: Sender<T>, T: From<LogMessage>> LoggingLayer<S, T> {
    pub const fn new(sender: S) -> Self {
        Self {
            sender,
            _ph: PhantomData,
        }
    }
}

#[derive(Default)]
struct FieldsVisitor(HashMap<String, serde_json::Value>);

impl Visit for FieldsVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().into(), format!("{value:?}").into());
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0.insert(field.name().into(), value.into());
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0.insert(field.name().into(), value.into());
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0.insert(field.name().into(), value.into());
    }

    fn record_i128(&mut self, field: &tracing::field::Field, value: i128) {
        self.record_debug(field, &value)
    }

    fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
        self.record_debug(field, &value)
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0.insert(field.name().into(), value.into());
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.insert(field.name().into(), value.into());
    }
}

impl<T: Sender<U> + 'static, U: From<LogMessage> + 'static, S: Subscriber> Layer<S>
    for LoggingLayer<T, U>
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let mut fields = FieldsVisitor::default();
        event.record(&mut fields);
        self.sender.send(
            LogMessage {
                level: match *metadata.level() {
                    tracing::Level::TRACE => gilbert_plugin_api::log::Level::Trace,
                    tracing::Level::DEBUG => gilbert_plugin_api::log::Level::Debug,
                    tracing::Level::INFO => gilbert_plugin_api::log::Level::Info,
                    tracing::Level::WARN => gilbert_plugin_api::log::Level::Warn,
                    tracing::Level::ERROR => gilbert_plugin_api::log::Level::Error,
                },
                name: metadata.name().into(),
                target: metadata.target().into(),
                module_path: metadata.module_path().map(Into::into),
                file: metadata.module_path().map(Into::into),
                line: metadata.line(),
                fields: fields.0,
            }
            .into(),
        )
    }
}
