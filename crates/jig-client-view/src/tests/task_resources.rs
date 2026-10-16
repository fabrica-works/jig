use super::*;

#[test]
fn task_resource_metadata_keeps_each_raw_slot_writer_presence_full_width_and_independent_flags() {
    let mut segments = List::with_capacity(CLIENT.rows);
    segments.push(Box::<[u8]>::from(*b"a/b")).unwrap();
    segments.push(Box::<[u8]>::from([255_u8; 32])).unwrap();
    let reference = OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(segments) };
    for writer in [
        None,
        Some(OwnerResourceWriter::Effect { entry: u64::MAX }),
        Some(OwnerResourceWriter::Run { task: u64::MAX, attempt: u64::MAX }),
    ] {
        for taken in [false, true] {
            let mut row = task_doc(&[]);
            row.resources = List::with_capacity(CLIENT.rows);
            for holder in [None, Some(u64::MAX)] {
                row.resources
                    .push(TaskResource::Owner(Box::new(OwnerResourceLine {
                        part: reference.clone(),
                        kind: u16::MAX,
                        taken,
                        holder,
                        writer,
                        writer_lost: !taken,
                    })))
                    .unwrap();
            }
            let (tree, bindings) = task_contract::page(row.clone());
            assert!(contains(&tree, b"Resource kind: 65535"));
            assert!(contains(&tree, b"Holder task: 18446744073709551615"));
            assert_eq!(contains(&tree, b"Resource taken"), taken);
            assert_eq!(contains(&tree, b"Resource not taken"), !taken);
            assert_eq!(contains(&tree, b"Writer lost"), !taken);
            assert_eq!(contains(&tree, b"Writer not lost"), taken);
            match writer {
                Some(OwnerResourceWriter::Run { .. }) => {
                    assert!(contains(&tree, b"Writer task: 18446744073709551615"));
                    assert!(contains(&tree, b"Writer attempt: 18446744073709551615"));
                    assert!(!contains(&tree, b"Writer effect:"));
                }
                Some(OwnerResourceWriter::Effect { .. }) => {
                    assert!(contains(&tree, b"Writer effect: 18446744073709551615"));
                    assert!(!contains(&tree, b"Writer task:"));
                }
                None => {
                    assert!(!contains(&tree, b"Writer effect:"));
                    assert!(!contains(&tree, b"Writer task:"));
                }
            }
            assert!(contains(&tree, b"a/b"));
            let escaped = "\\xFF".repeat(32);
            assert!(contains(&tree, escaped.as_bytes()));
            let mut slots = 0_u32;
            for node in tree.nodes() {
                if let Some(slot) = node.slot {
                    assert_eq!(bindings.owner_part(slot.token), Some(&reference));
                    assert_eq!(bindings.part(slot.token), None);
                    slots += 1;
                }
            }
            assert_eq!(slots, CLIENT.rows);
            assert!(tree.len() <= crate::task_nodes(&CLIENT, 0).unwrap());
            assert_eq!(row.resources.capacity(), CLIENT.rows);
        }
    }
}
