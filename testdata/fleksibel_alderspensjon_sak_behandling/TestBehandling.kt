package no.nav.pensjon.test

// Base interface
interface Behandling {
    fun getId(): String
}

// Another base class
open class BaseAktivitet {
    open fun execute() {}
}

interface AktivitetResponse

// Sample Behandling implementation
class FleksibelApSakBehandling(
    private val sakId: String
) : Behandling {
    override fun getId(): String = sakId

    override fun opprettInitiellAktivitet(): BaseAktivitet =
        VurderAktivitet()

    fun start() {
        val aktivitet = opprettInitiellAktivitet()
        aktivitet.execute()
    }
}

// Another class that extends Behandling
class AlderspensjonBehandling : Behandling {
    override fun getId(): String = "alderspensjon"
}

// Some unrelated class
class Helper {
    fun assist() {
        println("Helping")
    }
}

// A class extending BaseAktivitet
class VurderAktivitet : BaseAktivitet() {
    override fun execute() {
        println("Vurderer...")
    }
}

// Another aktivitet
class BehandleAktivitet : BaseAktivitet() {
    override fun execute() {
        println("Behandler...")
    }
}

class VentPaaDataAktivitet : BaseAktivitet() {
    override fun execute() {
        println("Venter på data...")
    }
}

class OpprettManuellOppgaveAktivitet : BaseAktivitet() {
    override fun execute() {
        println("Oppretter manuell oppgave...")
    }
}

class IverksettVedtakAktivitet : BaseAktivitet() {
    override fun execute() {
        println("Iverksetter vedtak...")
    }
}

class FullforKontrollAktivitet : BaseAktivitet() {
    override fun execute() {
        println("Fullfører kontroll...")
    }
}

// Processor base class
open class AktivitetProcessor<B : Behandling, A : BaseAktivitet> {
    open fun doProcess(behandling: B, aktivitet: A): AktivitetResponse? = null

    fun nesteAktivitet(aktivitet: BaseAktivitet): AktivitetResponse? = null
}

// AldeAktivitetProcessor - pattern where onFinished defines next step
open class AldeAktivitetProcessor<B : Behandling, A : BaseAktivitet> {
    open fun prosesserGrunnlag(behandling: B, aktivitet: A): Any? = null
    open fun prosesserVurdering(behandling: B, aktivitet: A, vurdering: Any?): Any? = null
    open fun onFinished(result: Any?): AktivitetResponse? = null

    fun nesteAktivitet(aktivitet: BaseAktivitet): AktivitetResponse? = null
}

// Test processors with doProcess pattern
class VurderAktivitetProcessor : AktivitetProcessor<FleksibelApSakBehandling, VurderAktivitet>() {
    override fun doProcess(behandling: FleksibelApSakBehandling, aktivitet: VurderAktivitet): AktivitetResponse? {
        val harData = behandling.getId().isNotEmpty()

        return if (harData) {
            nesteAktivitet(BehandleAktivitet())
        } else {
            nesteAktivitet(VentPaaDataAktivitet())
        }
    }
}

class BehandleAktivitetProcessor : AktivitetProcessor<FleksibelApSakBehandling, BehandleAktivitet>() {
    private val unleashNextService = UnleashService()

    override fun doProcess(behandling: FleksibelApSakBehandling, aktivitet: BehandleAktivitet): AktivitetResponse? {
        return when {
            unleashNextService.isEnabled("FEATURE_AUTO_BESLUTTER") -> nesteAktivitet(IverksettVedtakAktivitet())
            else -> nesteAktivitet(OpprettManuellOppgaveAktivitet())
        }
    }
}

class VentPaaDataAktivitetProcessor : AktivitetProcessor<FleksibelApSakBehandling, VentPaaDataAktivitet>() {
    override fun doProcess(behandling: FleksibelApSakBehandling, aktivitet: VentPaaDataAktivitet): AktivitetResponse? {
        return nesteAktivitet(BehandleAktivitet())
    }
}

// Test processor with onFinished pattern (like AldeAktivitetProcessor)
class OpprettManuellOppgaveAktivitetProcessor :
    AldeAktivitetProcessor<FleksibelApSakBehandling, OpprettManuellOppgaveAktivitet>() {
    override fun prosesserGrunnlag(
        behandling: FleksibelApSakBehandling,
        aktivitet: OpprettManuellOppgaveAktivitet
    ): Any? {
        // Some processing
        return null
    }

    override fun prosesserVurdering(
        behandling: FleksibelApSakBehandling,
        aktivitet: OpprettManuellOppgaveAktivitet,
        vurdering: Any?
    ): Any? {
        // Some vurdering
        return null
    }

    override fun onFinished(result: Any?): AktivitetResponse? {
        return nesteAktivitet(FullforKontrollAktivitet())
    }
}

class FullforKontrollAktivitetProcessor : AldeAktivitetProcessor<FleksibelApSakBehandling, FullforKontrollAktivitet>() {
    override fun onFinished(result: Any?): AktivitetResponse? {
        return nesteAktivitet(IverksettVedtakAktivitet())
    }
}

class IverksettVedtakAktivitetProcessor : AktivitetProcessor<FleksibelApSakBehandling, IverksettVedtakAktivitet>() {
    override fun doProcess(
        behandling: FleksibelApSakBehandling,
        aktivitet: IverksettVedtakAktivitet
    ): AktivitetResponse? {
        // End of flow - no next aktivitet
        return null
    }
}

class UnleashService {
    fun isEnabled(feature: String): Boolean = true
}
