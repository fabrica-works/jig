//! Semantic section proofs share the work-page fixture kit.
use super::*;
use crate::regions::JigRegion;
fn named(tree: &Tree, region: JigRegion) -> usize {
    tree.nodes().iter().position(|n| n.region == Some(region.region())).unwrap()
}
#[test]
fn settings_hold_their_five_named_regions() {
    let (d, _) = project(ProjectSection::Settings);
    for tree in [build(&d, 3, words()).0, super::super::owner_project::settings_tree()] {
        let outer = named(&tree, JigRegion::ProjectSettings);
        for (region, name) in [
            (JigRegion::ProjectResources, b"jig-project-resources".as_slice()),
            (JigRegion::ProjectPolicy, b"jig-project-policy".as_slice()),
            (JigRegion::ProjectRoles, b"jig-project-roles".as_slice()),
            (JigRegion::ProjectSpend, b"jig-project-spend".as_slice()),
            (JigRegion::ProjectRecurring, b"jig-project-recurring".as_slice()),
        ] {
            let index = named(&tree, region);
            assert_eq!(region.name(), name);
            assert_eq!(tree.nodes()[index].role, Role::Region, "{name:?}");
            assert!(index > outer && index < outer + usize::try_from(tree.nodes()[outer].size).unwrap(), "{name:?}");
            assert_eq!(tree.nodes()[index + 1].role, Role::Heading { level: 2 }, "{name:?}");
        }
    }
}
#[test]
fn the_goal_form_is_a_form_and_the_notes_search_a_search() {
    for (section, region, role, label) in [
        (ProjectSection::Goals, JigRegion::NewGoal, Role::Form, b"Goal specification".as_slice()),
        (ProjectSection::Notes, JigRegion::NotesSearch, Role::Search, b"Search notes by description".as_slice()),
    ] {
        let (d, _) = project(section);
        let (tree, _) = build(&d, 3, words());
        let index = named(&tree, region);
        assert_eq!(tree.nodes()[index].role, role);
        assert_eq!(tree.nodes()[index].name.as_deref(), Some(label));
        assert!(
            tree.nodes()[index + 1..index + usize::try_from(tree.nodes()[index].size).unwrap()]
                .iter()
                .any(|n| n.binding.is_some()),
            "{section:?}"
        );
    }
}
#[test]
fn each_project_section_carries_its_region_name() {
    for (section, region, name) in [
        (ProjectSection::Overview, JigRegion::ProjectOverview, b"jig-project-overview".as_slice()),
        (ProjectSection::Goals, JigRegion::ProjectGoals, b"jig-project-goals".as_slice()),
        (ProjectSection::Tasks, JigRegion::ProjectTasks, b"jig-project-tasks".as_slice()),
        (ProjectSection::Notes, JigRegion::ProjectNotes, b"jig-project-notes".as_slice()),
        (ProjectSection::Settings, JigRegion::ProjectSettings, b"jig-project-settings".as_slice()),
        (ProjectSection::Resources, JigRegion::ProjectResources, b"jig-project-resources".as_slice()),
        (ProjectSection::Policy, JigRegion::ProjectPolicy, b"jig-project-policy".as_slice()),
        (ProjectSection::Roles, JigRegion::ProjectRoles, b"jig-project-roles".as_slice()),
        (ProjectSection::Spend, JigRegion::ProjectSpend, b"jig-project-spend".as_slice()),
        (ProjectSection::Recurring, JigRegion::ProjectRecurring, b"jig-project-recurring".as_slice()),
    ] {
        let (d, _) = project(section);
        let (tree, _) = build(&d, 3, words());
        let index = named(&tree, region);
        assert_eq!(region.name(), name);
        assert_eq!(tree.nodes()[index].role, Role::Region, "{section:?}");
        assert_eq!(tree.nodes()[index + 1].role, Role::Heading { level: 2 });
    }
}

#[test]
fn the_project_tabs_mark_the_addresss_own_section_current() {
    for section in [
        ProjectSection::Overview,
        ProjectSection::Goals,
        ProjectSection::Tasks,
        ProjectSection::Notes,
        ProjectSection::Settings,
        ProjectSection::Resources,
        ProjectSection::Policy,
        ProjectSection::Roles,
        ProjectSection::Spend,
        ProjectSection::Recurring,
    ] {
        let (d, _) = project(section);
        let (tree, mut bindings) = build(&d, 3, words());
        assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Tab).count(), 10);
        let current: Vec<_> = tree
            .nodes()
            .iter()
            .filter(|n| n.role == Role::Tab && n.states.as_slice().contains(&crate::ui::State::Current))
            .collect();
        assert_eq!(current.len(), 1, "{section:?}");
        assert_eq!(
            crate::decode(&mut bindings, current[0].binding.unwrap().token, crate::Act::Press),
            Some(Act::Go { address: Address::Project { project: 4, section } })
        );
    }
}
