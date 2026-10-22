//! 把每条日志收成一行。sqlx 的 `db.statement` 自带换行，默认格式会把一条 SQL 拆成一大段。

use std::fmt;

use tracing::Subscriber;
use tracing::field::{Field, Visit};
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::format::{FormatEvent, FormatFields, Writer};
use tracing_subscriber::registry::LookupSpan;

use crate::support::time::now;

pub struct CompactLog;

pub fn init() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,tower_http=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .event_format(CompactLog)
        .init();
}

impl<S, N> FormatEvent<S, N> for CompactLog
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();
        let stamp = now().format("%H:%M:%S");
        write!(writer, "{stamp} {} {} ", meta.level(), meta.target())?;
        let mut fields = FlatFields::default();
        event.record(&mut fields);
        writeln!(writer, "{}", fields.0)
    }
}

#[derive(Default)]
struct FlatFields(String);

impl FlatFields {
    fn push(&mut self, name: &str, value: &str) {
        let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
        if value.is_empty() {
            return;
        }
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        if name == "message" {
            self.0.push_str(&value);
            return;
        }
        self.0.push_str(name);
        self.0.push('=');
        self.0.push_str(&value);
    }
}

impl Visit for FlatFields {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "db.statement" {
            return;
        }
        let rendered = format!("{value:?}");
        self.push(field.name(), rendered.trim_matches('"'));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "db.statement" {
            return;
        }
        self.push(field.name(), value);
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.push(field.name(), &value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.push(field.name(), &value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.push(field.name(), if value { "true" } else { "false" });
    }
}
