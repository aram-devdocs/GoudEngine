use super::*;

#[test]
fn test_add_dependency_basic() {
    let mut graph = DependencyGraph::new();
    let result = graph.add_dependency("shader.glsl", "common.glsl");

    assert!(result.is_ok());
    assert!(graph.contains("shader.glsl"));
    assert!(graph.contains("common.glsl"));
}

#[test]
fn test_add_dependency_records_forward_and_reverse() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("a.txt", "b.txt").unwrap();

    let deps = graph.get_dependencies("a.txt").unwrap();
    assert!(deps.contains("b.txt"));

    let rev = graph.get_dependents("b.txt").unwrap();
    assert!(rev.contains("a.txt"));
}

#[test]
fn test_self_dependency_is_cycle() {
    let mut graph = DependencyGraph::new();
    let result = graph.add_dependency("a.txt", "a.txt");

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.from, "a.txt");
    assert_eq!(err.to, "a.txt");
}

#[test]
fn test_direct_cycle_detected() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("a.txt", "b.txt").unwrap();
    let result = graph.add_dependency("b.txt", "a.txt");

    assert!(result.is_err());
}

#[test]
fn test_indirect_cycle_detected() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("a.txt", "b.txt").unwrap();
    graph.add_dependency("b.txt", "c.txt").unwrap();
    let result = graph.add_dependency("c.txt", "a.txt");

    assert!(result.is_err());
}

#[test]
fn test_cycle_error_display() {
    let err = CycleError {
        from: "a.txt".to_string(),
        to: "b.txt".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("a.txt"));
    assert!(msg.contains("b.txt"));
    assert!(msg.contains("cycle"));
}

#[test]
fn test_cascade_order_single_dependent() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("shader.glsl", "common.glsl").unwrap();

    let order = graph.get_cascade_order("common.glsl");
    assert_eq!(order, vec!["shader.glsl"]);
}

#[test]
fn test_cascade_order_chain() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("b.txt", "a.txt").unwrap();
    graph.add_dependency("c.txt", "b.txt").unwrap();

    let order = graph.get_cascade_order("a.txt");
    assert_eq!(order.len(), 2);
    assert_eq!(order[0], "b.txt");
    assert_eq!(order[1], "c.txt");
}

#[test]
fn test_cascade_order_no_dependents() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("a.txt", "b.txt").unwrap();

    let order = graph.get_cascade_order("a.txt");
    assert!(order.is_empty());
}

#[test]
fn test_cascade_order_diamond() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("b.txt", "a.txt").unwrap();
    graph.add_dependency("c.txt", "a.txt").unwrap();
    graph.add_dependency("d.txt", "b.txt").unwrap();
    graph.add_dependency("d.txt", "c.txt").unwrap();

    let order = graph.get_cascade_order("a.txt");
    assert_eq!(order.len(), 3);
    let d_pos = order.iter().position(|x| x == "d.txt").unwrap();
    let b_pos = order.iter().position(|x| x == "b.txt").unwrap();
    let c_pos = order.iter().position(|x| x == "c.txt").unwrap();
    assert!(b_pos < d_pos);
    assert!(c_pos < d_pos);
}

#[test]
fn test_cascade_excludes_changed_asset() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("b.txt", "a.txt").unwrap();

    let order = graph.get_cascade_order("a.txt");
    assert!(!order.contains(&"a.txt".to_string()));
}

#[test]
fn test_remove_asset_cleans_forward_edges() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("a.txt", "b.txt").unwrap();
    graph.remove_asset("a.txt");

    assert!(!graph.contains("a.txt"));
    assert!(graph.get_dependents("b.txt").is_none());
}

#[test]
fn test_remove_asset_cleans_reverse_edges() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("a.txt", "b.txt").unwrap();
    graph.remove_asset("b.txt");

    assert!(!graph.contains("b.txt"));
    assert!(graph.get_dependencies("a.txt").is_none());
}

#[test]
fn test_remove_asset_preserves_unrelated() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("a.txt", "b.txt").unwrap();
    graph.add_dependency("c.txt", "d.txt").unwrap();
    graph.remove_asset("a.txt");

    assert!(graph.contains("c.txt"));
    assert!(graph.contains("d.txt"));
}

#[test]
fn test_remove_nonexistent_is_noop() {
    let mut graph = DependencyGraph::new();
    graph.remove_asset("nonexistent.txt");
    assert_eq!(graph.asset_count(), 0);
}

#[test]
fn test_asset_count() {
    let mut graph = DependencyGraph::new();
    assert_eq!(graph.asset_count(), 0);

    graph.add_dependency("a.txt", "b.txt").unwrap();
    assert_eq!(graph.asset_count(), 2);

    graph.add_dependency("c.txt", "b.txt").unwrap();
    assert_eq!(graph.asset_count(), 3);
}

#[test]
fn test_clear() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("a.txt", "b.txt").unwrap();
    graph.add_dependency("c.txt", "d.txt").unwrap();
    graph.clear();

    assert_eq!(graph.asset_count(), 0);
}

#[test]
fn test_default() {
    let graph = DependencyGraph::default();
    assert_eq!(graph.asset_count(), 0);
}

#[test]
fn test_debug_format() {
    let graph = DependencyGraph::new();
    let debug = format!("{:?}", graph);
    assert!(debug.contains("DependencyGraph"));
}

#[test]
fn test_duplicate_dependency_is_idempotent() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("a.txt", "b.txt").unwrap();
    graph.add_dependency("a.txt", "b.txt").unwrap();

    let deps = graph.get_dependencies("a.txt").unwrap();
    assert_eq!(deps.len(), 1);
}

#[test]
fn test_multiple_dependencies_single_asset() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("shader.glsl", "utils.glsl").unwrap();
    graph
        .add_dependency("shader.glsl", "lighting.glsl")
        .unwrap();

    let deps = graph.get_dependencies("shader.glsl").unwrap();
    assert_eq!(deps.len(), 2);
    assert!(deps.contains("utils.glsl"));
    assert!(deps.contains("lighting.glsl"));
}

#[test]
fn test_non_cycle_allowed() {
    let mut graph = DependencyGraph::new();
    graph.add_dependency("a.txt", "b.txt").unwrap();
    graph.add_dependency("a.txt", "c.txt").unwrap();
    let result = graph.add_dependency("b.txt", "c.txt");
    assert!(result.is_ok());
}
