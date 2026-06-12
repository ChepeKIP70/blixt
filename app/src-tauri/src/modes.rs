// Die Arbeitsmodi (4 Original-Workflows aus blitztext-app + eigene Modi Übersetzen & Prompt).
// Die System-Prompts der 4 Original-Modi sind 1:1 portiert (BlitztextMac: LLMService.swift).
// Jeder Modus läuft: Aufnahme -> Transkription -> (optional) Textmodell.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Dictate,   // reine Transkription
    Improve,   // + sprachliche Aufpolierung (Lektorat)
    Vent,      // Frust -> klare, respektvolle Nachricht
    Emoji,     // + passende Emojis
    Translate, // DE <-> EN (Richtung automatisch erkannt)
    Prompt,    // grobe Absicht (DE) sprechen -> strukturierter KI-Prompt (EN)
    PromptDe,  // grobe Absicht (DE) sprechen -> strukturierter KI-Prompt (DE)
}

impl Mode {
    pub fn all() -> [Mode; 7] {
        [
            Mode::Dictate,
            Mode::Improve,
            Mode::Vent,
            Mode::Emoji,
            Mode::Translate,
            Mode::Prompt,
            Mode::PromptDe,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Mode::Dictate => "Diktat",
            Mode::Improve => "Verbessern",
            Mode::Vent => "Entschaerfen",
            Mode::Emoji => "Emoji",
            Mode::Translate => "Uebersetzen DE ↔ EN",
            Mode::Prompt => "Prompt (EN)",
            Mode::PromptDe => "Prompt (DE)",
        }
    }

    /// Statusanzeige während der Textmodell-Phase.
    pub fn processing_label(&self) -> &'static str {
        match self {
            Mode::Dictate => "Wird transkribiert ...",
            Mode::Improve => "Text wird verbessert ...",
            Mode::Vent => "Wird entschaerft ...",
            Mode::Emoji => "Emojis werden eingefuegt ...",
            Mode::Translate => "Wird uebersetzt ...",
            Mode::Prompt => "Prompt wird gebaut ...",
            Mode::PromptDe => "Prompt wird gebaut ...",
        }
    }

    /// Standard-Hotkey. Fn ist auf Windows nicht als Modifikator nutzbar -> Ctrl+Shift+Ziffer.
    pub fn default_hotkey(&self) -> &'static str {
        match self {
            Mode::Dictate => "Ctrl+Shift+1",
            Mode::Improve => "Ctrl+Shift+2",
            Mode::Vent => "Ctrl+Shift+3",
            Mode::Emoji => "Ctrl+Shift+4",
            Mode::Translate => "Ctrl+Shift+5",
            Mode::Prompt => "Ctrl+Shift+6",
            Mode::PromptDe => "Ctrl+Shift+7",
        }
    }

    /// Braucht dieser Modus nach der Transkription noch einen Textmodell-Durchlauf?
    pub fn needs_chat(&self) -> bool {
        !matches!(self, Mode::Dictate)
    }

    pub fn temperature(&self) -> f64 {
        match self {
            Mode::Dictate => 0.0,
            Mode::Improve => 0.3,
            Mode::Vent => 0.4,
            Mode::Emoji => 0.3,
            Mode::Translate => 0.3,
            Mode::Prompt => 0.4,
            Mode::PromptDe => 0.4,
        }
    }

    /// System-Prompt je Modus. 4 Original-Prompts 1:1 portiert; Übersetzen & Prompt sind eigen.
    pub fn system_prompt(&self) -> &'static str {
        match self {
            Mode::Dictate => "",
            Mode::Improve => "Du bist ein Lektor und Schreibassistent. Verbessere den folgenden Text:\n- Korrigiere Rechtschreibung und Grammatik\n- Verbessere die Formulierung und den Lesefluss\n- Behalte die ursprüngliche Bedeutung bei\n- Gib NUR den verbesserten Text zurück, keine Erklärungen\n- Verwende einen neutralen, klaren Ton",
            Mode::Vent => "Du erhältst ein emotional gesprochenes Transkript. Erkenne zuerst das eigentliche Ziel, Anliegen und den wahren Frust der Person. Formuliere daraus eine klare, respektvolle und wirksame Nachricht, mit der die Person ihr Ziel eher erreicht. Bewahre relevante Fakten, konkrete Probleme, Grenzen, Erwartungen und die nötige Dringlichkeit. Entferne Beleidigungen, Drohungen, Sarkasmus, Unterstellungen und unnötige Eskalation. Wenn mehrere Vorwürfe genannt werden, verdichte sie auf die entscheidenden Kernpunkte. Der Ton soll ruhig, menschlich, bestimmt und lösungsorientiert sein. Gib NUR die fertige Nachricht zurück.",
            Mode::Emoji => "Du erhältst ein gesprochenes Transkript. Gib den Text möglichst originalgetreu zurück, aber füge passende Emojis ein. Setze regelmäßig passende Emojis ein, etwa alle 1-2 Sätze. Korrigiere offensichtliche Sprach- und Grammatikfehler. Behalte den Stil und die Bedeutung bei. Gib NUR den Text mit Emojis zurück, keine Erklärungen.",
            Mode::Translate => "Du bist ein professioneller Übersetzer. Erkenne die Sprache des folgenden Textes: Ist er Deutsch, übersetze ihn in natürliches, flüssiges Englisch; ist er Englisch, übersetze ihn in natürliches, flüssiges Deutsch. Korrigiere offensichtliche Versprecher und Füllwörter. Bewahre Bedeutung, Tonfall und Absicht. Gib NUR die Übersetzung zurück, keine Erklärungen, keine Anmerkungen.",
            Mode::Prompt => "Du bist ein Prompt-Engineering-Assistent. Der Nutzer beschreibt auf Deutsch eine grobe Absicht oder Idee. Forme daraus einen klaren, gut strukturierten Prompt fuer ein KI-Sprachmodell, und zwar AUF ENGLISCH. Struktur: Rolle, Kontext, konkrete Aufgabe, relevante Randbedingungen und gewuenschtes Ausgabeformat - soweit aus der Eingabe ableitbar. Erfinde keine Fakten hinzu, die nicht gesagt wurden. Gib NUR den fertigen englischen Prompt zurueck, keine Vorrede, keine Erklaerungen.",
            Mode::PromptDe => "Du bist ein Prompt-Engineering-Assistent. Der Nutzer beschreibt eine grobe Absicht oder Idee. Forme daraus einen klaren, gut strukturierten Prompt fuer ein KI-Sprachmodell, AUF DEUTSCH. Struktur: Rolle, Kontext, konkrete Aufgabe, relevante Randbedingungen und gewuenschtes Ausgabeformat - soweit aus der Eingabe ableitbar. Erfinde keine Fakten hinzu, die nicht gesagt wurden. Gib NUR den fertigen deutschen Prompt zurueck, keine Vorrede, keine Erklaerungen.",
        }
    }
}
