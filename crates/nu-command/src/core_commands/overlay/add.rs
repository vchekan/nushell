use nu_engine::{eval_block, CallExt};
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{Category, Example, PipelineData, ShellError, Signature, Spanned, SyntaxShape};

use std::path::Path;

#[derive(Clone)]
pub struct OverlayAdd;

impl Command for OverlayAdd {
    fn name(&self) -> &str {
        "overlay add"
    }

    fn usage(&self) -> &str {
        "Add definitions from a module as an overlay"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build("overlay add")
            .required(
                "name",
                SyntaxShape::String,
                "Module name to create overlay for",
            )
            .optional(
                "as",
                SyntaxShape::Keyword(b"as".to_vec(), Box::new(SyntaxShape::String)),
                "as keyword followed by a new name",
            )
            .switch(
                "prefix",
                "Prepend module name to the imported commands and aliases",
                Some('p'),
            )
            .category(Category::Core)
    }

    fn extra_usage(&self) -> &str {
        r#"This command is a parser keyword. For details, check:
  https://www.nushell.sh/book/thinking_in_nu.html"#
    }

    fn is_parser_keyword(&self) -> bool {
        true
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let name_arg: Spanned<String> = call.req(engine_state, stack, 0)?;

        let (overlay_name, overlay_name_span) = if let Some(kw_expression) = call.positional_nth(1)
        {
            // If renamed via the 'as' keyword, use the new name as the overlay name
            if let Some(new_name_expression) = kw_expression.as_keyword() {
                if let Some(new_name) = new_name_expression.as_string() {
                    (new_name, new_name_expression.span)
                } else {
                    return Err(ShellError::NushellFailedSpanned(
                        "Wrong keyword type".to_string(),
                        "keyword argument not a string".to_string(),
                        new_name_expression.span,
                    ));
                }
            } else {
                return Err(ShellError::NushellFailedSpanned(
                    "Wrong keyword type".to_string(),
                    "keyword argument not a keyword".to_string(),
                    kw_expression.span,
                ));
            }
        } else if engine_state
            .find_overlay(name_arg.item.as_bytes())
            .is_some()
        {
            (name_arg.item, name_arg.span)
        } else if let Some(os_str) = Path::new(&name_arg.item).file_stem() {
            if let Some(name) = os_str.to_str() {
                (name.to_string(), name_arg.span)
            } else {
                return Err(ShellError::NonUtf8(name_arg.span));
            }
        } else {
            return Err(ShellError::OverlayNotFoundAtRuntime(
                name_arg.item,
                name_arg.span,
            ));
        };

        if let Some(overlay_id) = engine_state.find_overlay(overlay_name.as_bytes()) {
            let old_module_id = engine_state.get_overlay(overlay_id).origin;

            stack.add_overlay(overlay_name.clone());

            if let Some(new_module_id) = engine_state.find_module(overlay_name.as_bytes(), &[]) {
                if !stack.has_env_overlay(&overlay_name, engine_state)
                    || (old_module_id != new_module_id)
                {
                    // Add environment variables only if:
                    // a) adding a new overlay
                    // b) refreshing an active overlay (the origin module changed)
                    let module = engine_state.get_module(new_module_id);

                    for (name, block_id) in module.env_vars() {
                        let name = if let Ok(s) = String::from_utf8(name.clone()) {
                            s
                        } else {
                            return Err(ShellError::NonUtf8(call.head));
                        };

                        let block = engine_state.get_block(block_id);

                        let val = eval_block(
                            engine_state,
                            stack,
                            block,
                            PipelineData::new(call.head),
                            false,
                            true,
                        )?
                        .into_value(call.head);

                        stack.add_env_var(name, val);
                    }
                }
            }
        } else {
            return Err(ShellError::OverlayNotFoundAtRuntime(
                overlay_name,
                overlay_name_span,
            ));
        }

        Ok(PipelineData::new(call.head))
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Create an overlay from a module",
                example: r#"module spam { export def foo [] { "foo" } }
    overlay add spam
    foo"#,
                result: None,
            },
            Example {
                description: "Create an overlay with a prefix",
                example: r#"echo 'export def foo { "foo" }'
    overlay add --prefix spam
    spam foo"#,
                result: None,
            },
            Example {
                description: "Create an overlay from a file",
                example: r#"echo 'export env FOO { "foo" }' | save spam.nu
    overlay add spam.nu
    $env.FOO"#,
                result: None,
            },
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(OverlayAdd {})
    }
}
