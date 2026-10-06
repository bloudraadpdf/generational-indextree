use generational_indextree::{Arena, NodeId};

fn slot(node: NodeId) -> usize {
    generational_arena::Index::from(node).into_raw_parts().0
}

#[test]
fn a_node_releases_its_descendants_for_good() {
    let mut arena = Arena::new();
    let root = arena.new_node(0);
    let kept = arena.new_node(1);
    root.append(kept, &mut arena);
    let released = arena.new_node(2);
    root.append(released, &mut arena);
    let mut descendants = Vec::new();
    for value in 0..1_000 {
        let parent = descendants.get(value / 3).copied().unwrap_or(released);
        let child = arena.new_node(10 + value);
        parent.append(child, &mut arena);
        descendants.push(child);
    }
    let after = arena.new_node(3);
    root.append(after, &mut arena);
    let count = arena.count();

    let mut values = Vec::new();
    released.release_descendants(&mut arena, |value| values.push(value));

    values.sort_unstable();
    assert_eq!(values, (10..10 + descendants.len()).collect::<Vec<_>>());
    assert_eq!(arena.count(), count - descendants.len());
    assert!(arena.first_child(released).is_none());
    assert!(arena.last_child(released).is_none());
    assert!(descendants.iter().all(|&node| arena.get(node).is_none()));
    assert_eq!(root.children(&arena).collect::<Vec<_>>(), [kept, released, after]);
    let slots: Vec<_> = descendants.iter().copied().map(slot).collect();
    for value in 0..2_000 {
        assert!(!slots.contains(&slot(arena.new_node(value))));
    }
}
