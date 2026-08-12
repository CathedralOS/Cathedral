.[0] as $qualification
| .[1] as $outcomes
| .[2] as $contracts
| [
    $qualification.qualification_evidence[]
    | select(.domain == "Extent::Granted" and .origin == "admitted_receipt")
  ] as $admitted
| [
    $qualification.qualification_evidence[]
    | select(.domain == "Extent::Granted" and .origin == "propagated")
  ] as $propagated
| [
    $contracts.machines[]
    | select(.machine == "CathedralExtentRootProvider::grant_root")
  ] as $provider_contracts
| [
    $contracts.machines[]
    | select(.machine == "Main::main")
  ] as $boot_contracts
| [
    {
      name: "one receipt-backed provider crossing",
      ok: (
        ($admitted | length) == 2
        and ([$admitted[].subject] | sort) == [
          "CathedralExtentRootProvider::grant_root(geometry)",
          "Main::main::main::first_extent"
        ]
        and ([$admitted[].source] | unique) == ["ExtentRootProvider"]
        and ([$admitted[].requirement] | unique) == ["ExtentRootProvider::grant"]
        and ([$admitted[].receipt_identity] | unique | length) == 1
        and $admitted[0].receipt_identity != null
      )
    },
    {
      name: "propagated facts carry no fabricated receipts",
      ok: (
        ($propagated | length) == 1
        and all($propagated[];
          .program_point == "state"
          and .requirement == null
          and .receipt_identity == null
        )
      )
    },
    {
      name: "qualified root reaches the closed retain state",
      ok: any($propagated[];
        .subject == "Main::main::retain::extent"
        and .source == "Main::main::retain"
      )
    },
    {
      name: "provider returns the exact linear input claim",
      ok: (
        ($outcomes.claim_outcome_maps | length) == 1
        and ($outcomes.claim_outcome_maps[0].machine
          | startswith("CathedralExtentRootProvider::grant_root (#"))
        and $outcomes.claim_outcome_maps[0].state
          == "CathedralExtentRootProvider::grant_root::grant_root"
        and ($outcomes.claim_outcome_maps[0].entries | length) == 1
        and $outcomes.claim_outcome_maps[0].entries[0].output_path == []
        and $outcomes.claim_outcome_maps[0].entries[0].source.kind == "input"
        and ($outcomes.claim_outcome_maps[0].entries[0].source.parameter
          | startswith("root (#"))
        and $outcomes.claim_outcome_maps[0].entries[0].source.path == []
      )
    },
    {
      name: "granted extent has the canonical interval content projection",
      ok: (
        ($outcomes.content_projections | length) == 1
        and $outcomes.content_projections[0].domain == "Extent::Granted"
        and $outcomes.content_projections[0].carrier == "named(name(Extent))"
        and $outcomes.content_projections[0].projection_machine == "Granted::content"
        and $outcomes.content_projections[0].algebra == {
          kind: "interval_set",
          coordinate_space: "named(name(Nat))"
        }
        and $outcomes.content_projections[0].normalized_projection == {
          kind: "interval_set",
          members: [
            {
              start: {kind: "runtime_scalar_embedding", path: ["base"]},
              end: {
                kind: "arithmetic",
                operator: "add",
                left: {kind: "runtime_scalar_embedding", path: ["base"]},
                right: {kind: "runtime_scalar_embedding", path: ["length"]}
              }
            }
          ]
        }
      )
    },
    {
      name: "checked provider adapter is pure and terminating",
      ok: (
        ($provider_contracts | length) == 1
        and $provider_contracts[0].implementation.checked_may_suspend == false
        and $provider_contracts[0].implementation.checked_may_block == false
        and $provider_contracts[0].implementation.checked_service_reach == []
        and $provider_contracts[0].implementation.checked_termination.kind == "terminates"
      )
    },
    {
      name: "canary path retains the extent provider reach",
      ok: (
        ($boot_contracts | length) == 1
        and $boot_contracts[0].implementation.checked_may_suspend == false
        and $boot_contracts[0].implementation.checked_may_block == false
        and ($boot_contracts[0].implementation.checked_service_reach
          | index("ExtentRootProvider")) != null
      )
    }
  ]
| map(select(.ok | not) | .name) as $failures
| if ($failures | length) == 0 then
    true
  else
    error("qualified extent-root mismatch: " + ($failures | join(", ")))
  end
