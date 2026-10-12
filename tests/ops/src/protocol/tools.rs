//! Ops's five tool declarations and decoders, beside their schemas.
use super::json;
use jig_ops_domain_infrastructure as infra;
use jig_ops_domain_observability as obs;
use jig_protocol_charter::{Tool, ToolEffect};
use jig_protocol_tools::{Problem, Why};
use skein_lib::Duration;

pub enum Decoded {
    Read(obs::Read),
    Effect { effect: infra::Effect, reason: Option<Box<[u8]>> },
}

pub fn declarations(timeout: Duration) -> Vec<Tool> {
    [b"read_logs".as_slice(), b"read_series", b"read_alerts", b"effect_restart", b"propose_restart"]
        .into_iter()
        .map(|name| Tool {
            name: name.into(), description: name.into(),
            schema: if name == b"propose_restart" {
                br#"{"type":"object","properties":{"environment":{"type":"string"},"service":{"type":"string"},"operation":{"type":"integer"},"reason":{"type":"string"}},"required":["environment","service","operation","reason"]}"#.as_slice().into()
            } else if name == b"effect_restart" {
                br#"{"type":"object","properties":{"environment":{"type":"string"},"service":{"type":"string"},"operation":{"type":"integer"}},"required":["environment","service","operation"]}"#.as_slice().into()
            } else if name == b"read_logs" {
                br#"{"type":"object","properties":{"environment":{"type":"string"},"service":{"type":"string"},"from":{"type":"integer"},"through":{"type":"integer"},"max_bytes":{"type":"integer"},"filter":{"type":"string"}},"required":["environment","service","from","through","max_bytes","filter"]}"#.as_slice().into()
            } else {
                br#"{"type":"object","properties":{"environment":{"type":"string"},"service":{"type":"string"},"from":{"type":"integer"},"through":{"type":"integer"},"max_bytes":{"type":"integer"}},"required":["environment","service","from","through","max_bytes"]}"#.as_slice().into()
            },
            effect: if name.starts_with(b"read_") { ToolEffect::Read } else { ToolEffect::Write }, timeout,
        }).collect()
}

fn text(value: &json::Value, field: &'static [u8]) -> Result<Box<[u8]>, Problem> {
    value.required(field).and_then(json::Value::text).map_err(|why| Problem { field, why })
}
fn number(value: &json::Value, field: &'static [u8]) -> Result<u64, Problem> {
    value.required(field).and_then(json::Value::number).map_err(|why| Problem { field, why })
}

pub fn decode(tool: &[u8], input: &[u8]) -> Result<Decoded, Problem> {
    if !matches!(tool, b"read_logs" | b"read_series" | b"read_alerts" | b"effect_restart" | b"propose_restart") {
        return Err(Problem { field: b"tool", why: Why::Range });
    }
    let value = json::parse(input).map_err(|why| Problem { field: b"", why })?;
    let environment = text(&value, b"environment")?;
    let service = text(&value, b"service")?;
    match tool {
        b"effect_restart" | b"propose_restart" => Ok(Decoded::Effect {
            effect: infra::Effect::Restart {
                service: infra::Service::new(environment, service),
                operation: number(&value, b"operation")?,
            },
            reason: if tool == b"propose_restart" { Some(text(&value, b"reason")?) } else { None },
        }),
        b"read_logs" | b"read_series" | b"read_alerts" => {
            let service = obs::Service::new(environment, service);
            let window = obs::Window { from: number(&value, b"from")?, through: number(&value, b"through")? };
            let max_bytes = value
                .required(b"max_bytes")
                .and_then(json::Value::small)
                .map_err(|why| Problem { field: b"max_bytes", why })?;
            Ok(Decoded::Read(match tool {
                b"read_logs" => obs::Read::Logs { service, window, max_bytes, filter: text(&value, b"filter")? },
                b"read_series" => obs::Read::Series { service, window, max_bytes },
                b"read_alerts" => obs::Read::Fired { service, window, max_bytes },
                _ => unreachable!("matched read tool"),
            }))
        }
        _ => Err(Problem { field: b"tool", why: Why::Range }),
    }
}
