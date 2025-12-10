package no.nav.test

abstract class Behandling
abstract class Aktivitet
abstract class AktivitetProcessor<T : Aktivitet>

// Behandling with multiple separate cycles
class ComplexCycleBehandling : Behandling() {
    fun opprettInitiellAktivitet(): StartAktivitet {
        return StartAktivitet()
    }
}

// Activities
class StartAktivitet : Aktivitet()

// First cycle: Waiting loop
class VentPaaDataAktivitet : Aktivitet()
class SjekkDataAktivitet : Aktivitet()
class ReVentDataAktivitet : Aktivitet()

// Second cycle: Retry loop
class SendRequestAktivitet : Aktivitet()
class SjekkResponseAktivitet : Aktivitet()
class RetryRequestAktivitet : Aktivitet()

// Regular flow
class BehandleAktivitet : Aktivitet()
class AvsluttAktivitet : Aktivitet()

// Processors
class StartAktivitetProcessor : AktivitetProcessor<StartAktivitet>() {
    fun doProcess(aktivitet: StartAktivitet) {
        if (needsData()) {
            nesteAktivitet(VentPaaDataAktivitet())
        } else {
            nesteAktivitet(SendRequestAktivitet())
        }
    }
}

// First cycle: VentPaaData -> SjekkData -> ReVentData -> VentPaaData
class VentPaaDataAktivitetProcessor : AktivitetProcessor<VentPaaDataAktivitet>() {
    fun doProcess(aktivitet: VentPaaDataAktivitet) {
        nesteAktivitet(SjekkDataAktivitet())
    }
}

class SjekkDataAktivitetProcessor : AktivitetProcessor<SjekkDataAktivitet>() {
    fun doProcess(aktivitet: SjekkDataAktivitet) {
        if (dataReady()) {
            nesteAktivitet(BehandleAktivitet())
        } else {
            nesteAktivitet(ReVentDataAktivitet())
        }
    }
}

class ReVentDataAktivitetProcessor : AktivitetProcessor<ReVentDataAktivitet>() {
    fun doProcess(aktivitet: ReVentDataAktivitet) {
        // Cycle back to start of waiting loop
        nesteAktivitet(VentPaaDataAktivitet())
    }
}

// Second cycle: SendRequest -> SjekkResponse -> RetryRequest -> SendRequest
class SendRequestAktivitetProcessor : AktivitetProcessor<SendRequestAktivitet>() {
    fun doProcess(aktivitet: SendRequestAktivitet) {
        nesteAktivitet(SjekkResponseAktivitet())
    }
}

class SjekkResponseAktivitetProcessor : AktivitetProcessor<SjekkResponseAktivitet>() {
    fun doProcess(aktivitet: SjekkResponseAktivitet) {
        if (responseOk()) {
            nesteAktivitet(BehandleAktivitet())
        } else if (shouldRetry()) {
            nesteAktivitet(RetryRequestAktivitet())
        } else {
            nesteAktivitet(AvsluttAktivitet())
        }
    }
}

class RetryRequestAktivitetProcessor : AktivitetProcessor<RetryRequestAktivitet>() {
    fun doProcess(aktivitet: RetryRequestAktivitet) {
        // Cycle back to send request again
        nesteAktivitet(SendRequestAktivitet())
    }
}

// Regular processors
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
