// Explicit synthetic policy fixture, never registered by the production service.
// Adult Bronze source-bound definitions come from the core; this independent
// test policy admits only Lutz while retaining the Bronze spin/step mechanisms.
pub fn catalog() -> skating_planning_service::Catalog {
    use skating_program_lab::*;
    let mut catalog = skating_planning_service::Catalog::default();
    let mut profile = adult_profile(AdultCategory::Bronze);
    profile.id.name = "independent-http-policy".into();
    profile.category = "Independent HTTP test policy".into();
    profile.provenance = ProfileProvenance::Synthetic;
    profile
        .jumps
        .allowed
        .retain(|permission| permission.kind == JumpKind::Lutz);
    let mut policy = adult_program_policy(AdultCategory::Bronze);
    policy.spins.profile = profile.id.clone();
    policy.spins.source = profile.sources[0].clone();
    profile.sources.extend([
        policy.step_source.clone(),
        policy.choreography_source.clone(),
        policy.feature_source.clone(),
    ]);
    profile
        .sources
        .extend(policy.spins.positions.iter().map(|p| p.source.clone()));
    profile
        .sources
        .extend(policy.spins.per_foot.iter().map(|p| p.source.clone()));
    catalog
        .registry
        .register_with_program_policy(profile.clone(), policy)
        .unwrap();
    catalog.profiles.push(profile.clone());
    catalog
}
