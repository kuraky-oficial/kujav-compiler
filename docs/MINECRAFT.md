# KUJAV para plugins de Minecraft (Paper/Spigot)

## Qué soporta hoy

- Validación de JARs externos declarados en `[java].classpath`.
- Empaquetado de `plugin.yml` automático si configuras `[minecraft]`.

## Qué NO soporta todavía

- Generación completa de clases que extiendan `JavaPlugin`.
- Binding automático a eventos/commands de Bukkit/Paper.
- Resolución semántica completa de APIs Java externas.

Aun así, ya tienes base de proyecto y empaquetado para avanzar a ese objetivo.

## Configuración `kujav.toml` para plugin

```toml
[package]
name = "hola_plugin"
version = "0.1.0"
main = "src/main.kj"
edition = "2026"

[dependencies]

[java]
classpath = ["libs/paper-api-1.20.4-R0.1-SNAPSHOT.jar"]

[minecraft]
plugin_name = "HolaPlugin"
plugin_version = "0.1.0"
main_class = "com.tuorg.HolaPlugin"
api = "1.20"
```

## Archivo `.kj` mínimo

```kj
function main(): Int
    print "Kujav plugin build stub"
    return 0
end
```

## Build

```bash
kujav build
```

El JAR generado incluirá `plugin.yml` con los datos de `[minecraft]`.

## Usar cualquier librería Java (.jar/.class)

### Opción 1: JAR
1. Copia la dependencia en `libs/`.
2. Declárala en `[java].classpath`.
3. Ejecuta `kujav build`.

### Opción 2: clases sueltas
- Empaqueta tus `.class` en un JAR (recomendado) y usa la opción 1.

## Próximo paso recomendado para soporte real de plugins

1. Soporte de herencia `extends JavaPlugin` en AST + codegen.
2. Emisión de métodos `onEnable`/`onDisable`.
3. Llamadas de método con descriptor real (no fijo).
4. Resolver de tipos que use classpath Java para verificar firmas.
