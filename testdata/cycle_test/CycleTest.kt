package no.nav.test

abstract class Behandling
abstract class Aktivitet
abstract class AktivitetProcessor<T : Aktivitet>

// Main behandling with cycle
class CycleTestBehandling : Behandling() {
    fun opprettInitiellAktivitet(): StartAktivitet {
        return StartAktivitet()
    }
}

// Activities
class StartAktivitet : Aktivitet()
class VentPaaDataAktivitet : Aktivitet()
class SjekkDataAktivitet : Aktivitet()
class BehandleDataAktivitet : Aktivitet()
class AvsluttAktivitet : Aktivitet()

// Processors with cycle: VentPaaData -> SjekkData -> VentPaaData (waiting loop)
class StartAktivitetProcessor : AktivitetProcessor<StartAktivitet>() {
    fun doProcess(aktivitet: StartAktivitet) {
        nesteAktivitet(VentPaaDataAktivitet())
    }
}

class VentPaaDataAktivitetProcessor : AktivitetProcessor<VentPaaDataAktivitet>() {
    fun doProcess(aktivitet: VentPaaDataAktivitet) {
        nesteAktivitet(SjekkDataAktivitet())
    }
}

class SjekkDataAktivitetProcessor : AktivitetProcessor<SjekkDataAktivitet>() {
    fun doProcess(aktivitet: SjekkDataAktivitet) {
        if (dataErKlar()) {
            nesteAktivitet(BehandleDataAktivitet())
        } else {
            // Back to waiting - this creates a cycle!
            nesteAktivitet(VentPaaDataAktivitet())
        }
    }
}

class BehandleDataAktivitetProcessor : AktivitetProcessor<BehandleDataAktivitet>() {
    fun doProcess(aktivitet: BehandleDataAktivitet) {
        // End with aktivitetFullfort()
        aktivitetFullfort()
    }
}
