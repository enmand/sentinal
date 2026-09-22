use crate::queries::{Query, Statement};


pub(super) fn general_queries() -> Query {
    let v= vec![
        (
            "persistent_state",
            Statement::from("Does this change alter the stored representation, constraints, interpretation, lifecycle,
            mutation semantics, or deletion semantics of persistent application data, rather than only
            changing how existing data is read or queried?"),
        ),
        (
            "observable_behavior",
            Statement::from("Could this change alter behavior observable by users, callers, or other components, even
            if no public API or interface signature changes?"),
        ),
        (
            "external_contract",
            Statement::from("Does this change alter a documented or de facto external interface boundary, such as a
            public API signature, protocol, message format, event schema, serialized representation,
            command-line interface, or externally consumed configuration contract?"),
        ),
        (
            "security_boundary",
            Statement::from("Does this change alter authentication, authorization, identity, permissions, trust
            boundaries, credential handling, or another security-sensitive boundary?"),
        ),
        (
            "category",
            Statement::from((
                "Which category best describes the primary operational nature of this change?",
                [
                    (
                        "behavior_change", 
                        "Changes runtime behavior observable by users, callers, or other systems."
                    ),
                    (
                        "data_model_change",
                        "Changes persistent data structure, representation, constraints, interpretation, or lifecycle."
                    ),
                    (
                        "external_contract_change",
                        "Changes an API, protocol, event, message format, or another externally consumed interface."
                    ),
                    (
                        "infrastructure_change",
                        "Changes deployment, runtime infrastructure, networking, configuration, build, or operational environment."
                    ),
                    (
                        "security_change",
                        "Primarily changes authentication, authorization, identity, permissions, credentials, or trust boundaries."
                    ),
                    (
                        "internal_refactor",
                        "Changes internal implementation or code organization without intentionally changing observable runtime behavior."
                    ),
                    (
                        "observability_change",
                        "Primarily changes logging, metrics, tracing, monitoring, alerting, or diagnostics."
                    ),
                    (
                        "test_only",
                        "Changes only tests, fixtures, test data, or test infrastructure."
                    ),
                    (
                        "documentation_only",
                        "Changes only documentation, comments, examples, or other non-executable material."
                    ),
                ],
            ))
        ),
        (
            "blast_radius",
            Statement::from((
                "If this change contains a serious defect, how broad is the plausible production impact?",
                [
                    "No meaningful production impact.",
                    "Impact is limited to a narrow feature, code path, or small subset of users.",
                    "Impact could materially affect a major feature, service capability, or significant subset of users.",
                    "Impact could broadly affect the service, multiple components, or a large portion of users.",
                    "Impact could cause severe system-wide, security, financial, or data-integrity consequences.",
                ]
            ))
        ), 
        (
            "restore_complexity",
            Statement::from(("How operationally difficult would it be to restore the system to its pre-change state after this change has
             been deployed and exercised?",
            [
                "Redeploying the previous code is sufficient.",
                "Minor operational steps may be required, but rollback is routine.",
                "Rollback requires coordinated steps, migration handling, or careful verification.",
                "Production state may require repair, transformation, or cross-service coordination.",
                "The previous state may not be reliably recoverable because information or state may have been irreversibly changed.",
            ]))
        )];


    Query::new(v)
}
