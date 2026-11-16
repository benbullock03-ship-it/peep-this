# Architecture patterns

The names the Goose health report's candidate cards use. A card cites one `id`. Each entry: the lesson in one line, an ELI5, and where the idea comes from. The learned list (AV-110) reads this file.

| id | Name | Lesson | ELI5 | Source |
|---|---|---|---|---|
| `shallow-module` | Shallow module | A module whose interface is as big as what it hides costs more to learn than it saves. | A box with so many buttons you may as well do the job yourself. | Ousterhout, *A Philosophy of Software Design*, ch. 4 |
| `scattered-concept` | Scattered concept | One idea spread over many files means every change to it touches all of them. | Your socks are in five drawers, so getting dressed takes forever. | Ousterhout ch. 5 (information leakage); Pocock, codebase-design skill |
| `leaky-seam` | Leaky seam | When callers must know how a module works inside, the seam between them is not doing its job. | The kitchen hatch is so small the waiter has to come in and cook. | Parnas, "On the Criteria To Be Used in Decomposing Systems into Modules" (1972) |
| `pass-through` | Pass-through layer | A layer that only forwards calls adds a hop and hides nothing. | A messenger who just repeats what you said to the person next to you. | Ousterhout ch. 7 (pass-through methods) |
| `untested-core` | Untested core | Code many things depend on, with no test that can fail, makes every change a guess. | The bridge everyone drives over has never had a safety check. | Feathers, *Working Effectively with Legacy Code*, ch. 2 |
| `god-file` | God file | One file that owns many unrelated jobs grows until nobody can change it safely. | One drawer holding the cutlery, the batteries and the passports. | Riel, *Object-Oriented Design Heuristics*, heuristic 3.2 |

A candidate that fits none: the closest one, with a card note `pattern: closest fit`. Add a pattern here only with an owner's yes.
