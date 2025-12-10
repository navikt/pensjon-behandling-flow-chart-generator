class GenericTestBehandling : Behandling() {
    fun opprettInitiellAktivitet(): StartGenericActivity {
        return StartGenericActivity()
    }
}

class StartGenericActivity : Aktivitet()
class ProcessDataActivity : Aktivitet()
class ValidateItemActivity : Aktivitet()
class SaveResultActivity : Aktivitet()
class NotifyCompletionActivity : Aktivitet()
class CleanupActivity : Aktivitet()

class StartGenericActivityProcessor : AktivitetProcessor<StartGenericActivity>() {
    fun doProcess(aktivitet: StartGenericActivity): AktivitetResponse {
        return nesteAktivitet(ProcessDataActivity())
    }
}

class ProcessDataActivityProcessor : AktivitetProcessor<ProcessDataActivity>() {
    fun doProcess(aktivitet: ProcessDataActivity): AktivitetResponse {
        val items = getItemsToProcess()

        return if (shouldProcessInBatch()) {
            // Test case 1: Simple it.map pattern
            nesteAktiviteter(items.map { item -> ValidateItemActivity() })
        } else if (needsCleanup()) {
            // Test case 2: it.map + additional activity
            nesteAktiviteter(
                items.map { item -> ValidateItemActivity() } + CleanupActivity()
            )
        } else {
            // Test case 3: Multiple activities in list
            nesteAktiviteter(
                listOf(
                    SaveResultActivity(),
                    NotifyCompletionActivity(),
                    CleanupActivity()
                )
            )
        }
    }

    private fun getItemsToProcess(): List<String> = listOf("item1", "item2", "item3")
    private fun shouldProcessInBatch(): Boolean = true
    private fun needsCleanup(): Boolean = false
}

class ValidateItemActivityProcessor : AktivitetProcessor<ValidateItemActivity>() {
    fun doProcess(aktivitet: ValidateItemActivity): AktivitetResponse {
        return nesteAktivitet(SaveResultActivity())
    }
}

class SaveResultActivityProcessor : AktivitetProcessor<SaveResultActivity>() {
    fun doProcess(aktivitet: SaveResultActivity): AktivitetResponse {
        return aktivitetFullfort()
    }
}

class NotifyCompletionActivityProcessor : AktivitetProcessor<NotifyCompletionActivity>() {
    fun doProcess(aktivitet: NotifyCompletionActivity): AktivitetResponse {
        return aktivitetFullfort()
    }
}

class CleanupActivityProcessor : AktivitetProcessor<CleanupActivity>() {
    fun doProcess(aktivitet: CleanupActivity): AktivitetResponse {
        return aktivitetFullfort()
    }
}
