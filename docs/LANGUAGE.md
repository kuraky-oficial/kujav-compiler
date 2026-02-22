# KUJAV: guía del lenguaje (estado actual)

## Comandos CLI

- `kujav new <project>`
- `kujav build`
- `kujav run`
- `kujav check`
- `kujav clean`
- `kujav install` (placeholder)
- `kujav update` (placeholder)
- `kujav publish` (placeholder)

## Estructura de proyecto

```text
mi-proyecto/
├── kujav.toml
├── src/
│   └── main.kj
└── target/
```

## `kujav.toml`

```toml
[package]
name = "mi-proyecto"
version = "0.1.0"
main = "src/main.kj"
edition = "2026"

[dependencies]

[java]
classpath = ["libs/paper-api-1.20.4-R0.1-SNAPSHOT.jar"]
```

### Secciones soportadas

- `[package]`: metadatos obligatorios.
- `[dependencies]`: dependencias del ecosistema Kujav (lockfile determinista).
- `[java]`: `classpath` con JARs externos (se valida que existan en build).
- `[minecraft]`: activa generación de `plugin.yml` al empaquetar JAR.

## Sintaxis actual de `.kj`

```kj
function main(): Int
    local x: Int = 10
    print x
    return 0
end
```

> Nota: la sintaxis actual es estilo Lua (`function`, `local`, `end`) mientras se evoluciona al diseño final propuesto.

## Resultado de compilación

- `target/<nombre>.class`
- `target/<nombre>.jar`
- `kujav.lock`
