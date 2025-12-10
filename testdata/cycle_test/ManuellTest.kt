package no.nav.test

abstract class Behandling
abstract class Aktivitet
abstract class AktivitetProcessor<T : Aktivitet>
data class ManuellBehandling(val kategori: String, val beskrivelse: String)

// Behandling with manuellBehandling
class ManuellTestBehandling : Behandling() {
    fun opprettInitiellAktivitet(): StartAktivitet {
        return StartAktivitet()
    }
}

// Activities
class StartAktivitet : Aktivitet()
class VurderDataAktivitet : Aktivitet()
class BehandleAktivitet : Aktivitet()
class AvsluttAktivitet : Aktivitet()

// Processors
class StartAktivitetProcessor : AktivitetProcessor<StartAktivitet>() {
    fun doProcess(aktivitet: StartAktivitet) {
        nesteAktivitet(VurderDataAktivitet())
    }
}

class VurderDataAktivitetProcessor : AktivitetProcessor<VurderDataAktivitet>() {
    fun doProcess(aktivitet: VurderDataAktivitet) {
        if (needsManualReview()) {
            // This creates a manuellBehandling - should be marked with 📋
            manuellBehandling = ManuellBehandling(
                kategori = "MANUAL_REVIEW",
                beskrivelse = "Manual review required"
            )
            nesteAktivitet(BehandleAktivitet())
        } else {
            nesteAktivitet(BehandleAktivitet())
        }
    }
}

class BehandleAktivitetProcessor : AktivitetProcessor<BehandleAktivitet>() {
    fun doProcess(aktivitet: BehandleAktivitet) {
        nesteAktivitet(AvsluttAktivitet())
    }
}

class AvsluttAktivitetProcessor : AktivitetProcessor<AvsluttAktivitet>() {
    fun doProcess(aktivitet: AvsluttAktivitet) {
        aktivitetFullfort()
    }
}
