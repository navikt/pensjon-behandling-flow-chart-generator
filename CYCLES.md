# Cycle Detection in Behandling Flow Analyzer

This document explains how the tool detects and visualizes cycles (loops) in your Behandling flow.

## What is a Cycle?

A cycle occurs when the flow can return to an earlier aktivitet, creating a loop. This is common in:

- **Waiting states**: The flow waits for external data and repeatedly checks until it's ready
- **Retry logic**: The flow retries a failed operation multiple times
- **Polling mechanisms**: The flow polls an external service until a condition is met
- **Manual intervention loops**: The flow waits for manual actions and checks status

## How Cycles are Detected

The tool uses depth-first search (DFS) to detect back edges in the flow graph. A back edge is an edge that points to a node already in the current recursion stack, indicating a cycle.

### Example: Waiting Loop

```kotlin
class VentPaaDataAktivitetProcessor : AktivitetProcessor<VentPaaDataAktivitet>() {
    fun doProcess(aktivitet: VentPaaDataAktivitet) {
        nesteAktivitet(SjekkDataAktivitet())
    }
}

class SjekkDataAktivitetProcessor : AktivitetProcessor<SjekkDataAktivitet>() {
    fun doProcess(aktivitet: SjekkDataAktivitet) {
        if (dataErKlar()) {
            nesteAktivitet(BehandleDataAktivitet())  // Exit the cycle
        } else {
            nesteAktivitet(VentPaaDataAktivitet())  // Back to waiting - creates a cycle!
        }
    }
}
```

This creates a cycle: `VentPaaData → SjekkData → VentPaaData`

## Visual Representation

### In the Graph

Cycles are highlighted with several visual cues:

1. **Cluster Box**: All nodes in a cycle are enclosed in a red dashed box
2. **Background**: The cluster has a light pink background (#FFF5F5)
3. **Label**: The box is labeled "🔄 Waiting/Retry Loop"
4. **Back Edges**: Edges that create the cycle are shown in bold red
5. **Constraint**: Back edges use `constraint=false` to prevent them from affecting layout

### Example DOT Output

```dot
subgraph cluster_0 {
  style="rounded,dashed";
  color="#FF6B6B";
  penwidth=2.5;
  bgcolor="#FFF5F5";
  label="🔄 Waiting/Retry Loop";
  fontcolor="#FF6B6B";
  fontsize=12;
  fontname="Arial Bold";
  "VentPaaDataAktivitet";
  "SjekkDataAktivitet";
}

"SjekkDataAktivitet" -> "VentPaaDataAktivitet" [
  label="NOT (dataReady())",
  color="#FF6B6B",
  penwidth=2,
  style=bold,
  constraint=false
];
```

### In Verbose Output

When running with `--verbose`, the tool reports detected cycles:

```
Flow for MyBehandling:
  Starting with: StartAktivitet
    → VentPaaDataAktivitet
      → SjekkDataAktivitet
        → [IF dataReady()] BehandleAktivitet
        → [ELSE] VentPaaDataAktivitet
          [CYCLE DETECTED: VentPaaDataAktivitet]

  🔄 Detected 1 cycle(s) in this flow:
    SjekkData ↩ VentPaaData
```

## Multiple Cycles

The tool can detect and group multiple independent cycles in the same flow.

### Example: Separate Waiting and Retry Loops

```kotlin
// First cycle: Data waiting
VentPaaData → SjekkData → VentPaaData

// Second cycle: Request retry
SendRequest → CheckResponse → RetryRequest → SendRequest
```

Each cycle gets its own cluster:

```
subgraph cluster_0 {
  label="🔄 Waiting/Retry Loop";
  // First cycle nodes
}

subgraph cluster_1 {
  label="🔄 Waiting/Retry Loop";
  // Second cycle nodes
}
```

## Common Cycle Patterns

### 1. Simple Wait Loop

The most common pattern - wait for data or event:

```kotlin
Wait → Check → Wait (if not ready)
              → Continue (if ready)
```

### 2. Retry with Backoff

Retry a failing operation:

```kotlin
Execute → CheckResult → Retry (if failed)
                      → Success (if succeeded)
                      → GiveUp (if max retries)
```

### 3. Manual Intervention Loop

Wait for manual action:

```kotlin
CreateTask → CheckTask → WaitForTask (if not done)
                       → Continue (if done)
```

### 4. Multi-Step Cycle

Cycle involving multiple steps:

```kotlin
A → B → C → D → A (back to start)
```

## Cycle Detection Algorithm

The tool uses the following algorithm:

1. **Depth-First Search (DFS)**: Traverse the flow graph starting from the initial aktivitet
2. **Recursion Stack**: Keep track of nodes in the current path
3. **Back Edge Detection**: When we encounter a node already in the recursion stack, we've found a cycle
4. **Grouping**: Use connected component analysis to group nodes that participate in cycles
5. **Cluster Creation**: Generate Graphviz subgraph clusters for each group

### Algorithm Details

```rust
fn detect_cycles(start: &str, processor_index: &HashMap<String, ProcessorInfo>) -> Vec<(String, String)> {
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    
    fn dfs(node: &str, visited: &mut HashSet<String>, rec_stack: &mut HashSet<String>, cycles: &mut Vec<(String, String)>) {
        visited.insert(node);
        rec_stack.insert(node);
        
        for next in get_next_nodes(node) {
            if rec_stack.contains(next) {
                // Back edge found - this is a cycle!
                cycles.push((node, next));
            } else if !visited.contains(next) {
                dfs(next, visited, rec_stack, cycles);
            }
        }
        
        rec_stack.remove(node);
    }
    
    dfs(start, &mut visited, &mut rec_stack, &mut cycles);
    cycles
}
```

## Why Cycles Matter

Cycles are important to identify because they:

1. **Indicate waiting states**: Show where the flow pauses for external events
2. **Reveal retry logic**: Highlight error handling and recovery mechanisms
3. **Show potential infinite loops**: Help identify places where flow might get stuck
4. **Document business logic**: Make it clear that certain operations are repeated

## Visual Design Choices

### Why Red Dashed Box?

- **Red**: Commonly associated with attention/warning - cycles need careful design
- **Dashed**: Distinguishes from solid boxes, indicates "special" grouping
- **Bold**: Makes the boundary clearly visible
- **Rounded**: Matches the overall soft, modern design of the graph

### Why Light Pink Background?

- Subtle enough not to overwhelm
- Distinguishes cycle nodes from non-cycle nodes
- Works well with all node colors
- Maintains readability of text

### Why Bold Red Back Edges?

- Clearly shows which edge creates the cycle
- Distinguishes from forward edges
- Uses same color as cluster for consistency
- `constraint=false` prevents layout distortion

## Best Practices

### For Flow Design

1. **Always have exit conditions**: Ensure every cycle has a way to exit
2. **Limit cycle depth**: Avoid cycles that go through too many nodes
3. **Document timeout logic**: Make sure cycles have timeout/max-retry limits
4. **Test cycle exit**: Verify that exit conditions are reachable

### For Using the Tool

1. **Use verbose mode**: See detailed cycle information with `--verbose`
2. **Check cycle edges**: Look at the bold red edges to understand loop conditions
3. **Verify exit paths**: Ensure each cycle cluster has edges leading out
4. **Keep DOT files**: Use `--keep-dot` to inspect the structure manually

## Troubleshooting

### "Too many cycles detected"

If the tool reports many cycles, it might indicate:
- Complex retry logic with multiple layers
- Nested waiting states
- Multiple independent polling mechanisms
- Potential design issues (too many loops)

### "Cycle cluster looks wrong"

If nodes are grouped incorrectly:
- The flow might have overlapping cycles
- Check the verbose output to see which back edges were detected
- Use `--keep-dot` to inspect the DOT file
- Consider simplifying the flow logic

### "Missing cycle edges"

If a cycle exists but isn't detected:
- The processor might be missing
- The edge might go through an unknown node
- Check for dynamic dispatch or complex control flow

## Examples

### Example 1: Simple Waiting Loop

**Input Flow:**
```
Start → VentPaaData → SjekkData → (if ready) Behandle → End
                         ↑             |
                         └─────────────┘ (if not ready)
```

**Visual Output:**
```
┌─────────────────────────────────┐
│ 🔄 Waiting/Retry Loop           │
│                                 │
│  ┌──────────┐    ┌──────────┐  │
│  │VentPaaData│───→│SjekkData │  │
│  └──────────┘    └──────────┘  │
│       ↑               │         │
│       └───────────────┘ (red)  │
└─────────────────────────────────┘
```

### Example 2: Multiple Cycles

**Input Flow:**
```
Start → (branch A) Cycle1 → Continue
      → (branch B) Cycle2 → End
```

**Visual Output:**
```
┌─────────────────┐        ┌─────────────────┐
│ 🔄 Loop 1       │        │ 🔄 Loop 2       │
│   VentData      │        │   SendRequest   │
│   SjekkData     │        │   CheckResponse │
└─────────────────┘        └─────────────────┘
```

## Technical Notes

- Cycles are detected using Tarjan's algorithm for finding back edges
- Connected components are found using DFS
- Graphviz clusters are named `cluster_0`, `cluster_1`, etc.
- Back edges use `constraint=false` to avoid affecting graph layout
- The tool handles self-loops (node pointing to itself) as single-node cycles

## See Also

- [README.md](README.md) - Main documentation
- [USAGE.md](USAGE.md) - Usage guide with examples
- [BINARY_USAGE.md](BINARY_USAGE.md) - CLI reference