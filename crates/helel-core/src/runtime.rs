//! Local inference artifact validation and supervised runtime protocol.

use crate::local_process::JsonLineProcess;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io, path::PathBuf, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelArtifacts {
    pub config: PathBuf,
    pub tokenizer: PathBuf,
    pub weights: PathBuf,
}

impl ModelArtifacts {
    /// Validates compatible local artifact metadata before process startup.
    ///
    /// # Errors
    /// Returns an error when an artifact is absent, malformed, empty, or incompatible.
    pub fn validate(&self) -> io::Result<()> {
        let config: serde_json::Value =
            serde_json::from_slice(&fs::read(&self.config)?).map_err(io::Error::other)?;
        let tokenizer: serde_json::Value =
            serde_json::from_slice(&fs::read(&self.tokenizer)?).map_err(io::Error::other)?;
        let model_vocab = config
            .get("vocabulary_size")
            .or_else(|| config.get("vocabularySize"))
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "model vocabulary size missing")
            })?;
        let tokenizer_vocab = tokenizer
            .get("vocabulary_size")
            .or_else(|| tokenizer.get("vocabularySize"))
            .and_then(serde_json::Value::as_u64)
            .or_else(|| {
                tokenizer
                    .get("vocabulary")
                    .and_then(serde_json::Value::as_array)
                    .map(|v| v.len() as u64)
            })
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "tokenizer vocabulary size missing",
                )
            })?;
        if model_vocab != tokenizer_vocab {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "model and tokenizer vocabulary sizes differ",
            ));
        }
        if fs::metadata(&self.weights)?.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "weights file is empty",
            ));
        }
        let checkpoint = self
            .weights
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("checkpoint.json");
        if checkpoint.is_file() {
            let metadata: serde_json::Value =
                serde_json::from_slice(&fs::read(checkpoint)?).map_err(io::Error::other)?;
            let filename = self
                .weights
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "invalid weights path")
                })?;
            if let Some(expected) = metadata
                .get("files")
                .and_then(|files| files.get(filename))
                .and_then(serde_json::Value::as_str)
            {
                let actual = format!("{:x}", Sha256::digest(fs::read(&self.weights)?));
                if actual != expected {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "model weights checksum mismatch",
                    ));
                }
            }
        }
        Ok(())
    }
}

pub struct LocalModelRuntime {
    process: JsonLineProcess,
    maximum_new_tokens: usize,
}
impl LocalModelRuntime {
    /// Starts a packaged or explicitly selected local inference executable.
    ///
    /// # Errors
    /// Returns an error when artifacts fail validation, startup fails, or health-check fails.
    pub fn start(
        program: &str,
        base_args: &[String],
        artifacts: &ModelArtifacts,
    ) -> io::Result<Self> {
        artifacts.validate()?;
        let config: serde_json::Value =
            serde_json::from_slice(&fs::read(&artifacts.config)?).map_err(io::Error::other)?;
        let maximum_new_tokens = config
            .get("context_length")
            .or_else(|| config.get("contextLength"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(128)
            .clamp(1, 512) as usize;
        let mut args = base_args.to_vec();
        for (flag, path) in [
            ("--config", &artifacts.config),
            ("--tokenizer", &artifacts.tokenizer),
            ("--weights", &artifacts.weights),
        ] {
            args.push(flag.into());
            args.push(path.to_string_lossy().into_owned());
        }
        let process = JsonLineProcess::start(program, &args)?;
        let response = process.request(
            r#"{"kind":"health","request_id":"startup"}"#,
            Duration::from_secs(10),
        )?;
        let value: serde_json::Value = serde_json::from_str(&response).map_err(io::Error::other)?;
        if value.get("kind").and_then(serde_json::Value::as_str) != Some("healthy") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "local runtime health check failed",
            ));
        }
        Ok(Self {
            process,
            maximum_new_tokens,
        })
    }
    #[must_use]
    pub const fn maximum_new_tokens(&self) -> usize {
        self.maximum_new_tokens
    }
    /// Sends a validated generation request and returns the next event.
    ///
    /// # Errors
    /// Returns transport or JSON validation errors.
    pub fn generate(&self, json: &str) -> io::Result<Vec<serde_json::Value>> {
        self.process.send(json)?;
        let mut events = Vec::new();
        loop {
            let value: serde_json::Value =
                serde_json::from_str(&self.process.receive(Duration::from_secs(60))?)
                    .map_err(io::Error::other)?;
            let terminal = matches!(
                value.get("kind").and_then(serde_json::Value::as_str),
                Some("done" | "error")
            );
            events.push(value);
            if terminal {
                break;
            }
        }
        Ok(events)
    }
    /// Stops the runtime.
    ///
    /// # Errors
    /// Returns an error when termination fails.
    pub fn stop(&self) -> io::Result<()> {
        self.process.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_empty_weights_and_mismatch() {
        let d = tempfile::tempdir().unwrap();
        let c = d.path().join("c.json");
        let t = d.path().join("t.json");
        let w = d.path().join("w");
        fs::write(&c, r#"{"vocabulary_size":2}"#).unwrap();
        fs::write(&t, r#"{"vocabulary_size":3}"#).unwrap();
        fs::write(&w, []).unwrap();
        assert!(
            ModelArtifacts {
                config: c.clone(),
                tokenizer: t.clone(),
                weights: w.clone()
            }
            .validate()
            .is_err()
        );
        fs::write(&t, r#"{"vocabulary_size":2}"#).unwrap();
        fs::write(&w, [1]).unwrap();
        fs::write(
            d.path().join("checkpoint.json"),
            r#"{"files":{"w":"0000000000000000000000000000000000000000000000000000000000000000"}}"#,
        )
        .unwrap();
        assert!(
            ModelArtifacts {
                config: c,
                tokenizer: t,
                weights: w
            }
            .validate()
            .unwrap_err()
            .to_string()
            .contains("checksum")
        );
    }
    #[test]
    fn supervises_health_and_streaming_protocol() {
        let d = tempfile::tempdir().unwrap();
        let config = d.path().join("config.json");
        let tokenizer = d.path().join("tokenizer.json");
        let weights = d.path().join("model.safetensors");
        fs::write(&config, r#"{"vocabulary_size":2}"#).unwrap();
        fs::write(&tokenizer, r#"{"vocabulary_size":2}"#).unwrap();
        fs::write(&weights, [1]).unwrap();
        let code = "import sys,json\nfor line in sys.stdin:\n p=json.loads(line); rid=p.get('request_id','x')\n if p.get('kind')=='health': print(json.dumps({'request_id':rid,'kind':'healthy'}),flush=True)\n else:\n  print(json.dumps({'request_id':rid,'kind':'token','text':'ok'}),flush=True)\n  print(json.dumps({'request_id':rid,'kind':'done'}),flush=True)";
        let runtime = LocalModelRuntime::start(
            "python3",
            &["-u".into(), "-c".into(), code.into()],
            &ModelArtifacts {
                config,
                tokenizer,
                weights,
            },
        )
        .unwrap();
        let events = runtime
            .generate(r#"{"request_id":"g","prompt":"p"}"#)
            .unwrap();
        assert_eq!(events[0]["text"], "ok");
        runtime.stop().unwrap();
    }
}
