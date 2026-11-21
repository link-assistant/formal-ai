use super::*;

/// Parse execution configuration without reading process-global state.
///
/// A runner is usable only when its isolation is explicitly `docker`.
/// Supplying half of the pair is an error, never permission to fall back to
/// host execution. An explicit `host_sandbox` value is the only configuration
/// that selects the host backend.
pub fn backend_from_configuration(
    start_isolation: Option<&str>,
    start_runner: Option<&str>,
    backend: Option<&str>,
) -> Result<Option<ExecutionBackend>, BoxError> {
    let isolation = start_isolation
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let runner = start_runner
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match (isolation, runner) {
        (Some(isolation), Some(runner)) => {
            if isolation != "docker" {
                return Err(BoxError::InvalidConfiguration {
                    detail: ["unsupported start isolation", isolation].join(": "),
                });
            }
            let words =
                shell_words::split(runner).map_err(|error| BoxError::InvalidConfiguration {
                    detail: error.to_string(),
                })?;
            let Some((program, arguments)) = words.split_first() else {
                return Err(BoxError::InvalidConfiguration {
                    detail: String::from("the start runner is empty"),
                });
            };
            return Ok(Some(ExecutionBackend::StartRunner {
                program: program.clone(),
                arguments: arguments.to_vec(),
                isolation: isolation.to_owned(),
            }));
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(BoxError::InvalidConfiguration {
                detail: String::from("start isolation and runner must be configured together"),
            });
        }
        (None, None) => {}
    }

    let Some(backend) = backend.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if backend == "host_sandbox" {
        return Ok(Some(ExecutionBackend::HostSandbox));
    }
    if let Some(language) = backend
        .strip_prefix("box-language:")
        .filter(|language| !language.is_empty())
    {
        let contract = crate::box_language_projects::box_language_contract();
        if let Some(project) = contract
            .projects
            .iter()
            .find(|project| project.language == language)
        {
            return Ok(Some(ExecutionBackend::Box {
                image: format!("{}:{}", project.image, contract.image_tag),
            }));
        }
        if let Some(deferred) = contract
            .deferred
            .iter()
            .find(|project| project.language == language)
        {
            return Err(BoxError::InvalidConfiguration {
                detail: deferred.reason.clone(),
            });
        }
        return Err(BoxError::InvalidConfiguration {
            detail: ["no box-language contract for", language].join(": "),
        });
    }
    if let Some(image) = backend
        .strip_prefix("box:")
        .filter(|image| !image.is_empty())
    {
        validate_image(image)?;
        return Ok(Some(ExecutionBackend::Box {
            image: image.to_owned(),
        }));
    }
    Err(BoxError::InvalidConfiguration {
        detail: ["unknown execution backend", backend].join(": "),
    })
}
