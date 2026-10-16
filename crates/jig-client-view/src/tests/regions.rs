use crate::regions::JigRegion;

#[test]
fn every_jig_region_has_its_designed_name() {
    let rows: &[(JigRegion, &[u8])] = &[
        (JigRegion::Link, b"jig-link"),
        (JigRegion::Needs, b"jig-needs"),
        (JigRegion::Party, b"jig-party"),
        (JigRegion::Projects, b"jig-projects"),
        (JigRegion::GoToTask, b"jig-go-to-task"),
        (JigRegion::Navigation, b"jig-navigation"),
        (JigRegion::Owed, b"jig-owed"),
        (JigRegion::Confirmation, b"jig-confirmation"),
        (JigRegion::Notices, b"jig-notices"),
        (JigRegion::InboxHeader, b"jig-inbox-header"),
        (JigRegion::InboxNeeds, b"jig-inbox-needs"),
        (JigRegion::InboxUpdates, b"jig-inbox-updates"),
        (JigRegion::Conversations, b"jig-conversations"),
        (JigRegion::TaskHeader, b"jig-task-header"),
        (JigRegion::TaskTabs, b"jig-task-tabs"),
        (JigRegion::TaskOverview, b"jig-task-overview"),
        (JigRegion::TaskActivity, b"jig-task-activity"),
        (JigRegion::TaskPlan, b"jig-task-plan"),
        (JigRegion::TaskHistory, b"jig-task-history"),
        (JigRegion::TaskBudget, b"jig-task-budget"),
        (JigRegion::TaskPanels, b"jig-task-panels"),
        (JigRegion::ProjectHeader, b"jig-project-header"),
        (JigRegion::ProjectTabs, b"jig-project-tabs"),
        (JigRegion::ProjectOverview, b"jig-project-overview"),
        (JigRegion::ProjectGoals, b"jig-project-goals"),
        (JigRegion::ProjectTasks, b"jig-project-tasks"),
        (JigRegion::ProjectNotes, b"jig-project-notes"),
        (JigRegion::ProjectSettings, b"jig-project-settings"),
        (JigRegion::ProjectResources, b"jig-project-resources"),
        (JigRegion::ProjectPolicy, b"jig-project-policy"),
        (JigRegion::ProjectRoles, b"jig-project-roles"),
        (JigRegion::ProjectSpend, b"jig-project-spend"),
        (JigRegion::ProjectRecurring, b"jig-project-recurring"),
        (JigRegion::Deployment, b"jig-deployment"),
        (JigRegion::Conversation, b"jig-conversation"),
        (JigRegion::Composer, b"jig-composer"),
        (JigRegion::NewChat, b"jig-new-chat"),
        (JigRegion::NewGoal, b"jig-new-goal"),
        (JigRegion::NotesSearch, b"jig-notes-search"),
        (JigRegion::Hosts, b"jig-hosts"),
        (JigRegion::Accounts, b"jig-accounts"),
        (JigRegion::Connectors, b"jig-connectors"),
        (JigRegion::Capacity, b"jig-capacity"),
        (JigRegion::Starting, b"jig-starting"),
        (JigRegion::SigningIn, b"jig-signing-in"),
        (JigRegion::Missing, b"jig-missing"),
        (JigRegion::OutOfDate, b"jig-out-of-date"),
        (JigRegion::SomeoneElse, b"jig-someone-else"),
    ];
    for (region, name) in rows {
        assert_eq!(region.name(), *name, "{region:?}");
    }
}

#[test]
fn a_region_name_survives_its_number() {
    assert_eq!(JigRegion::of(JigRegion::Link.region().number), Some(JigRegion::Link));
    assert_eq!(JigRegion::of(JigRegion::TaskPlan.region().number), Some(JigRegion::TaskPlan));
    assert_eq!(JigRegion::of(JigRegion::SomeoneElse.region().number), Some(JigRegion::SomeoneElse));
    assert_eq!(JigRegion::of(48), None);
}
