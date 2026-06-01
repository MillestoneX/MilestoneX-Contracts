#[test]
fn returns_active_status() {
    let env = Env::default();

    let result =
        CampaignContract::get_campaign_status(
            env.clone(),
        );

    assert_eq!(
        result.status,
        CampaignStatus::Active
    );

    assert!(result.days_remaining > 0);
}

#[test]
fn returns_successful_status() {
    let env = Env::default();

    // setup campaign
    // raised >= goal

    let result =
        CampaignContract::get_campaign_status(
            env.clone(),
        );

    assert_eq!(
        result.status,
        CampaignStatus::Successful
    );
}
#[test]
fn returns_failed_status() {
    let env = Env::default();

    // deadline passed
    // goal not reached

    let result =
        CampaignContract::get_campaign_status(
            env.clone(),
        );

    assert_eq!(
        result.status,
        CampaignStatus::Failed
    );

    assert!(result.days_remaining < 0);
}

#[test]
fn returns_cancelled_status() {
    let env = Env::default();

    // mark cancelled

    let result =
        CampaignContract::get_campaign_status(
            env.clone(),
        );

    assert_eq!(
        result.status,
        CampaignStatus::Cancelled
    );
}
#[test]
fn calculates_days_remaining() {
    let env = Env::default();

    let result =
        CampaignContract::get_campaign_status(
            env.clone(),
        );

    assert!(
        result.days_remaining >= -3650
    );
}