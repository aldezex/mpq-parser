# mpq-parser

Parser en Rust puro para el contenedor de archivos **MPQ (Mo'PaQ)**, el formato de archivo genérico usado por varios juegos de Blizzard Entertainment (StarCraft II, WarCraft III, World of Warcraft, Diablo) para empaquetar sus assets y datos.

Escrito **desde cero**, sin apoyarse en crates de parsing MPQ existentes — el objetivo principal es servir de proyecto de aprendizaje de Rust (parsing binario, manejo de errores, diseño de librería), no competir en funcionalidad con implementaciones más maduras del ecosistema (`wow-mpq`, `ceres-mpq`).

## Origen

Este crate nació como parte de [sc2reader-rs](https://github.com/aldezex/sc2reader-rs), un port de aprendizaje de [sc2reader](https://github.com/ggtracker/sc2reader) (Python) a Rust. Como el formato MPQ no es específico de StarCraft II, se extrajo a su propia librería independiente — con su propio ciclo de versionado y publicación — en vez de mantenerlo acoplado a un proyecto que sí es específico de un juego.

## Estado actual

🚧 Cubre únicamente el **header** del contenedor MPQ:
- [x] `MPQUserData` (el envoltorio que precede al header MPQ real en archivos como los replays de SC2).
- [x] `MpqHeader` — header MPQ real, formato V1-V4 (campos básicos: tamaños, versión, posiciones y tamaños de hash table / block table).
- [ ] Lectura y desencriptación del contenido de la **hash table**.
- [ ] Lectura del contenido de la **block table**.
- [ ] Extracción y descompresión de archivos internos.

No soporta (todavía, y puede que nunca, dado el alcance de aprendizaje del proyecto): escritura/creación de archivos MPQ, versiones de protocolo muy antiguas, archivos protegidos/firmados.

## Uso

```rust
use mpq_parser::{MpqUserDataHeader, MpqHeader};

let bytes = std::fs::read("replay.SC2Replay")?;

let user_header = MpqUserDataHeader::parse(&bytes)?;
let offset = user_header.header_offset as usize;
let mpq_header = MpqHeader::parse(&bytes[offset..])?;

println!("{:?}", mpq_header);
```

## Manejo de errores

Todas las funciones de parsing devuelven `Result<T, MpqParseError>` en vez de hacer panic — el llamador decide qué hacer ante datos corruptos o incompletos.

## Licencia

Dual licenciado bajo MIT o Apache-2.0, a elección de quien lo use. Ver `LICENSE-MIT.md` y `LICENSE-APACHE.md`.

## Contribuciones

Proyecto personal de aprendizaje — no se buscan activamente contribuciones externas, pero issues y sugerencias son bienvenidas.
