# Accepted language and terminology contract

Accepted on 2026-11-09 for [Define multilingual terminology and comprehension responsibilities](https://github.com/tizian-bitschi/skating-program-lab/issues/150). The user confirmed all three live proposals with “3 mal ja”. This is a planning decision, not an implementation or completed language review.

## Language selection

Support German (`de`), English (`en`) and Russian (`ru`). Prefer the language saved with the local workspace. Otherwise use a supported browser language; if none is supported, fall back to English. Manual switching remains available at any time and updates the saved preference.

Language changes preserve the program, repertoire restrictions and active search. They do not change the planning context/revision or trigger a new optimization. Display text and numeric formatting follow the selected locale; canonical codes, exact values and request identities do not change.

## Names, codes and findings

Translate category/element names, controls, state labels and rule/error explanations. Keep international codes such as CSp, CCSp and 1A unchanged. Preserve exact spin distinctions, including Camel versus Layback, flying entry and foot change. Official rule sources remain linked in their original editions; translating the interface does not rewrite the rules.

Keep the accepted C2 interaction direction: short meaningful labels and concrete errors, with additional help available when needed. Compact codes retain accessible full names and the agreed focus/inspection behavior. Do not add permanent explanatory paragraphs to compensate for ambiguous controls.

Service diagnostics use the accepted stable codes/translation keys and parameters. The UI renders the current language rather than displaying Rust Debug output. Personal restrictions, invalid input, category violations, proven maximum and exhausted collection must remain distinguishable in all three languages.

## Translation ownership and review

The agent prepares the initial three language catalogs. The user performs or organizes domain-language review against concrete screens; the agent applies corrections directly to the catalogs. No separate editorial system is required.

Record which languages, screens and terms were actually reviewed and any unresolved issues in the acceptance evidence. Initial translations, complete key coverage or the user's approval of this workflow do not establish native-language approval or participant comprehension. Actual language/usability checks belong to the remaining acceptance decision and later implementation verification.

## Boundaries

No UI, storage or service code was changed by this decision. The existing prototype language handling is not a substitute for implementing and verifying this contract. The agreed age-12+/adult and desktop/phone scope remains unchanged.
