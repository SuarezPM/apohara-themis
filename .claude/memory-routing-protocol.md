# Memory Routing Protocol — MOIRAI v4 (Band of Agents Hackathon)

> Inyectado en cada sesión vía hook `SessionStart`. **Una sola capa activa** (engram).
> qdrant y cognee fueron retirados/diferidos en jun 2026 — el rol de "búsqueda
> semántica" lo cubre la búsqueda EXA en runtime, no una capa de memoria local.
> El rol de "grafo de relaciones" lo cubre la navegación GitNexus + lectura del
> código del workspace.

## engram — CRONOLOGÍA y SESIONES (el "diario", única capa activa)

- **Qué guarda:** decisiones, bugs + causa raíz, convenciones, preferencias del
  usuario, resúmenes de sesión. Responde "¿qué hicimos / cuándo / por qué?".
- **Tools:** `mem_save`, `mem_search`, `mem_context`, `mem_session_summary`.
- **Project detection:** engram deduce el proyecto del `cwd` (git root).
  - Sesión actual: `~/apohara-moirai/` → proyecto `apohara_moirai`
  - Sesión anterior: `~/Documentos/Apohara_Synthex/` → proyecto `apohara_synthex`
  - **Cross-project search**: `mem_search(all_projects=true)` busca en todas.
- **Dispara STORE:** tras CADA decisión / bug / descubrimiento (proactivo, ya
  obligatorio por el protocolo engram); y al cerrar sesión (`mem_session_summary`).
- **Dispara SEARCH:** cuando Pablo pide recordar algo histórico, o al retomar
  trabajo previo. **Al arrancar sesión en MOIRAI, buscá primero en `apohara_moirai`,
  después en `all_projects=true` si no encontrás.**
- **NO uses engram para:** contenido largo verbatim (→ leer el archivo directo)
  ni relaciones entre entidades (→ GitNexus).

## Project routing para MOIRAI v4

| Contexto | Dónde buscar |
|---|---|
| "¿Qué specs cristalizamos para el hackathon?" | `mem_search(project="apohara_moirai", "spec v4 / 18 ACs / workstreams")` |
| "¿Qué aprendimos de Synthex v2?" | `mem_search(project="apohara_synthex", "Synthex v2 post-mortem")` o BOOTSTRAP.md |
| "¿Cómo se relacionan MOIRAI y el cerebro?" | CLAUDE.md §8 (acuerdo explícito) |
| "Specs iterativos descartados" | `~/apohara-hackathon-brain/` filesystem (no engram) |
| "Competidores / papers / news" | `~/apohara-hackathon-brain/research/raw-notes/` filesystem |
| "Deep dive Band SDK / chat rooms" | `~/apohara-hackathon-brain/docs/03-research/band-deep-dive.md` |

## Reglas de sesión para este workspace

1. **El spec está en `docs/SPEC.md`.** Es la fuente de verdad. Si Pablo pregunta
   "¿qué construimos?", leo el spec, no mem_search.
2. **El cerebro está en `~/apohara-hackathon-brain/`.** Lo abro cuando Pablo
   pregunta "¿qué hizo ATLAS/ATRIO?" o "¿hay papers nuevos sobre X?".
3. **Las 7 reglas en `.claude/rules/`** auto-cargan. Si no las estoy siguiendo,
   el sistema no me lo va a recordar — me las tengo que leer yo.
4. **El contrato brain↔MOIRAI está en `CLAUDE.md` §8.** Si una decisión del
   cerebro cambia el spec, actualizo el spec. El cerebro no es la fuente de verdad.
5. **Cero código antes del kickoff** (jueves 12 jun 12:00 UYT). Si Pablo pide
   código antes, señalo que estamos en pre-kickoff.
