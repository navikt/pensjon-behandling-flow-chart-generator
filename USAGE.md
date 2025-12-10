# Behandling Flow Analyzer - Usage Guide

## Quick Start

```bash
# Analyze a Kotlin project directory
cargo run -- /path/to/kotlin/project

# Use default test directory
cargo run
```

## What It Does

This tool analyzes Kotlin codebases to:

1. **Find Behandling classes** - Identifies main treatment classes
2. **Extract initial activities** - Discovers the starting point (`opprettInitiellAktivitet()`)
3. **Map the flow** - Traces through all aktivitet processors
4. **Show conditional branches** - Extracts actual if/else conditions from code
5. **Generate diagrams** - Creates DOT graph for visualization

## Example Output

### Summary
```
Scanning directory: /path/to/project
Scanned 54 .kt files
Indexed 122 classes
Found 27 processors

=== SUMMARY ===

Main Behandling classes with initial aktivitet:

  FleksibelApSakBehandling (FleksibelApSakBehandling.kt)
    → opprettInitiellAktivitet() returns: FleksibelApSakA101SettBehandlingstypeAutoAktivitet
```

### Flow Traversal
```
=== AKTIVITET FLOW ===

Flow for FleksibelApSakBehandling:
  Starting with: FleksibelApSakA101SettBehandlingstypeAutoAktivitet
    → FleksibelApSakA102KontrollerInformasjonsgrunnlagAktivitet
      → [IF krav.regelverkType.isAlderspensjon2025] FleksibelApSakA103OpprettLivsvarigAfpOffentligGrunnlagAktivitet
        → [PROCESSOR NOT FOUND]
      → [IF NOT (krav.regelverkType.isAlderspensjon2025)] FleksibelApSakA201SettBrukPaRettPersongrunnlagAktivitet
        → FleksibelApSakA301VurderOgLagreVurdertSivilstandAktivitet
          → FleksibelApSakA400VurderInstitusjonsoppholdAktivitet
            → FleksibelApSakA401VurderUtlandsOppholdAktivitet
              → ...
```

### DOT Graph Generation
```
=== GENERATING DOT GRAPH ===

Generated: FleksibelApSakBehandling_flow.dot
  To visualize: dot -Tpng FleksibelApSakBehandling_flow.dot -o FleksibelApSakBehandling.png
               dot -Tsvg FleksibelApSakBehandling_flow.dot -o FleksibelApSakBehandling.svg
```

## Visualizing the Diagram

### Method 1: Graphviz (Local)

Install Graphviz:
```bash
# macOS
brew install graphviz

# Ubuntu/Debian
sudo apt-get install graphviz

# Windows
choco install graphviz
```

Generate images:
```bash
# PNG (raster)
dot -Tpng FleksibelApSakBehandling_flow.dot -o flow.png

# SVG (vector, recommended)
dot -Tsvg FleksibelApSakBehandling_flow.dot -o flow.svg

# PDF
dot -Tpdf FleksibelApSakBehandling_flow.dot -o flow.pdf
```

### Method 2: VS Code

Install "Graphviz Preview" extension, then open the `.dot` file.

## Understanding the Diagram

### Node Colors

- 🟢 **Green (START)** - Entry point of the flow
- 🟣 **Purple** - AldeAktivitet (important activities with grunnlag/vurdering pattern)
- 🔵 **Sky Blue** - Regular processing activities
- 🟡 **Gold** - Waiting/pause activities (names contain "Vent" or "Wait")
- 🔴 **Red** - Manual intervention required (contains "Manuell", "Oppgave")
- 🟥 **Dark Red** - Abort/rejection (contains "Avbryt", "Avslag")
- 🟩 **Green** - Decision/execution (contains "Vedtak", "Iverksett")
- 🩷 **Pink (END)** - Terminal nodes
- ⚪ **Gray (?)** - Unknown/missing processors
- 🟠 **Orange (📋)** - Creates manual task/oppgave (manuellBehandling)

### Cycle Detection

The tool automatically detects and highlights cycles (loops) in the flow:

- **Visual grouping**: Nodes involved in a cycle are enclosed in a red dashed box
- **Cluster label**: "🔄 Waiting/Retry Loop" appears at the top of each cycle
- **Back edges**: Edges that create the cycle are shown in **bold red**
- **Multiple cycles**: Each separate cycle gets its own cluster
- **Light background**: Cycle clusters have a subtle pink background (#FFF5F5)

Common patterns detected as cycles:
- **Waiting loops**: VentPaaData → SjekkData → VentPaaData (waiting for external data)
- **Retry logic**: SendRequest → CheckResponse → RetryRequest → SendRequest
- **Polling states**: Any activity that loops back to itself or earlier activities

Example in code:
```kotlin
// This creates a cycle
class SjekkDataProcessor : AktivitetProcessor<SjekkDataAktivitet>() {
    fun doProcess(aktivitet: SjekkDataAktivitet) {
        if (dataReady()) {
            nesteAktivitet(BehandleAktivitet())  // Exit cycle
        } else {
            nesteAktivitet(VentPaaDataAktivitet())  // Back to waiting (cycle!)
        }
    }
}
```

### Manual Task Detection

The tool detects when an aktivitet creates a manual task (oppgave):

- **Pattern detected**: `manuellBehandling = ManuellBehandling(...)`
- **Visual indicator**: 📋 emoji in node label
- **Color**: Orange (#FFA500)

Example in code:
```kotlin
class VurderSamboerProcessor : AktivitetProcessor<VurderSamboerAktivitet>() {
    fun doProcess(aktivitet: VurderSamboerAktivitet) {
        if (needsManualReview()) {
            // This creates a manual task - node will be marked with 📋
            manuellBehandling = ManuellBehandling(
                kategori = FleksibelApSakBehandlingManuelleKategorier.SAMBOER,
                beskrivelse = "Manual review required",
                aktivTil = LocalDate.now().plusDays(7),
                oppgaveCode = OppgaveCode.KRA,
            )
            nesteAktivitet(NextAktivitet())
        }
    }
}
```

This helps identify where in the flow manual intervention is triggered.

### Edge Labels

Edges show the conditions from the actual Kotlin code:

```
A → B [label="krav.regelverkType.isAlderspensjon2025"]
A → C [label="NOT (krav.regelverkType.isAlderspensjon2025)"]
A → D [label="🚩 FEATURE: PEN_VURDER_SAMBOER && erEgnetForAlde(behandling, krav)"]
```

This means:
- If condition is true → go to B
- If condition is false → go to C
- If feature toggle is enabled AND condition → go to D

**Feature Toggles**: When a condition involves `unleashNextService.isEnabled()`, it's marked with a 🚩 flag emoji and shows the feature flag name clearly.

## Code Patterns Detected

### 1. Simple Linear Flow

```kotlin
override fun doProcess(...): AktivitetResponse {
    // ... processing logic ...
    return nesteAktivitet(NextActivity())
}
```

Output: `CurrentActivity → NextActivity`

### 2. Conditional Branch

```kotlin
override fun doProcess(...): AktivitetResponse {
    return if (condition) {
        nesteAktivitet(ActivityA())
    } else {
        nesteAktivitet(ActivityB())
    }
}
```

Output:
```
CurrentActivity → [IF condition] ActivityA
CurrentActivity → [IF NOT (condition)] ActivityB
```

### 3. Complex Conditions

```kotlin
override fun doProcess(...): AktivitetResponse {
    return when {
        feature.isEnabled() && krav.isValid() -> nesteAktivitet(A())
        !krav.isValid() -> nesteAktivitet(B())
        else -> nesteAktivitet(C())
    }
}
```

Output shows all branches with their full conditions.

## What Gets Extracted

### From Behandling Classes
- Class name
- Supertypes (inheritance hierarchy)
- `opprettInitiellAktivitet()` method → initial activity class name

### From Processor Classes
- Class name (must end with "Processor")
- Associated aktivitet class (from generic type parameter)
- `doProcess()` method → all `nesteAktivitet()` calls
- Conditional logic → extracted from if/when expressions

## Limitations

### Missing Processors
If you see `[PROCESSOR NOT FOUND]`, it means:
- The aktivitet class doesn't have a corresponding processor in the scanned directory
- The processor is in a different module/directory
- The processor uses a different naming pattern

### Complex Control Flow
- Dynamic dispatch (interface calls) is not traced
- Lambdas passed as parameters are not analyzed
- Some complex Kotlin expressions may be simplified in conditions

### Cycles
- The tool detects cycles to prevent infinite loops
- Cycles are shown as `[CYCLE DETECTED]` in text output
- In diagrams, cycles create back-edges

## Tips

### Focus on Specific Flows
To analyze only the main behandling flow, look for classes that:
- Extend `Behandling` (not just any subclass)
- Have `opprettInitiellAktivitet()` defined
- Are marked as `[MAIN]` in the output

### Reading Complex Branches
When you see multiple branches from one node:
1. Look for the first `[IF condition]` - this is the primary path
2. `[IF NOT (condition)]` is the alternative
3. `[ELSE]` appears when there are multiple conditions or when/switch cases

### Finding Bottlenecks
Look for:
- 🟡 **Gold nodes** (Vent) - where the flow pauses
- 🔴 **Red nodes** (Manuell/Oppgave) - where manual intervention is needed
- Many branches → complex decision points

## Troubleshooting

### No processors found
- Check that you're pointing to the correct directory
- Ensure processor classes follow the naming pattern `*Processor`
- Verify that processors extend the expected base class with type parameters

### Conditions look wrong
The tool extracts conditions verbatim from the AST. Some may be:
- Truncated (if longer than 80 chars)
- Simplified (common prefixes like `behandling.` and `krav.` are removed)
- Feature toggles are formatted specially with 🚩 emoji
- Check the actual Kotlin code for full details

### Many overlapping arrows
The tool **automatically consolidates** multiple conditional paths between the same nodes:

**What it does:**
- When 4+ conditions lead from node A to node B, shows: `"N alternative paths"` with bold styling
- When 2-3 conditions exist, shows count with example: `"3 alternative paths (e.g. condition1)"`
- Single conditions are shown in full

**Example:**
```dot
# Instead of 7 separate arrows:
"A601" -> "A999" [label="condition1"];
"A601" -> "A999" [label="condition2"];
...
"A601" -> "A999" [label="condition7"];

# You get one consolidated arrow:
"A601" -> "A999" [label="7 alternative paths", style=bold, penwidth=2];
```

**For more detail:**
If you need to see the individual conditions, check the text output which shows all branches, or inspect the source `.kt` file for the specific processor.

### Graph too large
For very large flows:
- Use SVG output (scales better than PNG)
- Increase DPI: `dot -Tpng -Gdpi=300 input.dot -o output.png`
- Use subgraph clustering (manual DOT editing)
- Split into multiple subgraphs for different parts of the flow

## Advanced Usage

### Custom Diagram Layout

Edit the generated `.dot` file to customize:

```dot
// Horizontal layout instead of vertical
rankdir=LR;

// Different node shapes
node [shape=rect];

// Colored edges
edge [color=blue];

// Cluster related nodes
subgraph cluster_0 {
    label = "Step 1: Validation";
    A101; A102; A103;
}
```

### Filtering Output

Pipe to grep for specific patterns:
```bash
cargo run -- /path/to/project 2>&1 | grep "A4"  # Only step 4 activities
cargo run -- /path/to/project 2>&1 | grep "IF"  # Only conditional branches
```

## Example: Real-World Flow

For `FleksibelApSakBehandling`, the tool discovered:
- **54 Kotlin files** in the directory
- **122 total classes** indexed
- **27 processors** with flow logic
- **1 main Behandling class** with initial aktivitet
- **Multiple conditional branches** based on feature flags, krav type, etc.

The generated diagram shows the complete flow from initial request through:
1. Setting treatment type
2. Validating information
3. Checking control points
4. Calculating benefits
5. Creating decision
6. Executing/completing

All conditional paths are visible, making it easy to:
- Understand the full process
- Find where manual intervention occurs
- Identify waiting points
- See feature flag impacts