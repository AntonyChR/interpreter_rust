# Interpreter (Rust)

Intérprete de lenguaje de programación, implementado desde cero en Rust. Incluye lexer, parser Pratt, AST, evaluador tree-walking, entornos con scopes y un REPL interactivo.

El proyecto está inspirado en el diseño de *Writing an Interpreter in Go* de Thorsten Ball, pero la implementación está escrita íntegramente en Rust y no utiliza dependencias externas.

## Características

- Lexer con soporte para identificadores, enteros, strings, operadores y delimitadores.
- Parser Pratt con precedencia de operadores.
- AST tipado para statements y expresiones.
- Evaluador tree-walking.
- Variables con `let`.
- Funciones, llamadas, retorno y closures.
- Condicionales `if` / `else`.
- Valores booleanos, enteros de 64 bits, strings y arrays.
- Concatenación de strings con `+`.
- Entornos anidados con scopes léxicos.
- Builtin `len()` para strings.
- REPL interactivo.
- 30 pruebas automatizadas.

## Tour rápido

El REPL evalúa una línea a la vez. Las funciones deben definirse en una sola línea.

```text
>> let x = 10;
>> let double = fn(n) { n * 2; };
>> double(x);
20
```

Funciones y strings:

```text
>> let greet = fn(name) { "hola, " + name; };
>> greet("mundo");
hola, mundo
```

Condicionales:

```text
>> if (10 > 5) { "mayor"; } else { "menor"; }
mayor
```

Arrays:

```text
>> let numbers = [1, 2, 3];
>> numbers;
[1, 2, 3]
```

Closures:

```text
>> let adder = fn(x) { fn(y) { x + y; }; };
>> let addTen = adder(10);
>> addTen(5);
15
```

Builtin `len`:

```text
>> len("hello");
5
```

## Gramática

La gramática soportada actualmente es:

```ebnf
program             = statement* EOF ;
statement           = letStatement | returnStatement | expressionStatement ;
letStatement        = "let" identifier "=" expression ";"? ;
returnStatement     = "return" expression? ";"? ;
expressionStatement = expression ;

expression          = equality ;
equality            = comparison (("==" | "!=") comparison)* ;
comparison          = term (("<" | ">") term)* ;
term                = factor (("+" | "-") factor)* ;
factor              = unary (("*" | "/") unary)* ;
unary               = ("!" | "-") unary | call ;
call                = primary ("(" arguments? ")")* ;

primary             = identifier
                    | integer
                    | string
                    | "true"
                    | "false"
                    | "(" expression ")"
                    | ifExpression
                    | functionLiteral
                    | arrayLiteral ;

ifExpression        = "if" "(" expression ")" block ("else" block)? ;
functionLiteral     = "fn" "(" parameters? ")" block ;
arrayLiteral        = "[" (expression ("," expression)*)? "]" ;
block               = "{" statement* "}" ;

parameters          = identifier ("," identifier)* ;
arguments           = expression ("," expression)* ;
```

## Requisitos

- Rust estable con soporte para edición 2021 (Rust 1.56 o superior).
- Cargo.
- `git` es opcional: se usa en `build.rs` para mostrar el hash corto del commit en el banner. Si no está disponible, se muestra `unknown`.

No se requieren crates externos.

## Compilar y ejecutar

Clonar el repositorio y ejecutar el REPL:

```bash
git clone https://github.com/AntonyChR/interpreter_rust.git
cd interpreter_rust
cargo run
```

Compilar en modo release:

```bash
cargo build --release
./target/release/interpreter
```

Para salir del REPL, usar `Ctrl+C`.

## Pruebas

Ejecutar toda la suite:

```bash
cargo test
```

El proyecto incluye 30 pruebas unitarias distribuidas entre lexer, parser, AST, evaluador, entorno y utilidades.

## Estructura del proyecto

```text
.
├── build.rs              # Inyecta el hash corto del commit en el binario
├── Cargo.toml
├── src
│   ├── main.rs           # Punto de entrada y banner
│   ├── token.rs          # Tipos de token y palabras reservadas
│   ├── lexer.rs          # Análisis léxico
│   ├── ast.rs            # Nodos del AST
│   ├── parser.rs         # Parser Pratt
│   ├── object.rs         # Valores en tiempo de ejecución
│   ├── environment.rs    # Entornos y scopes
│   ├── evaluator.rs      # Evaluador tree-walking
│   ├── builtins.rs       # Funciones nativas
│   ├── repl.rs           # REPL interactivo
│   └── utils.rs          # Helpers de caracteres
└── test.sh               # Ejecuta una prueba puntual del evaluador
```

## Notas de implementación

- El lexer y el parser trabajan sobre `&str` y conservan referencias con el lifetime `'a`, evitando copiar el código fuente en cada token.
- El parser usa funciones prefix/infix registradas por tipo de token, con una tabla de precedencias para expresiones.
- El evaluador recorre el AST y produce valores del enum `Object`.
- Los entornos se comparten mediante `Rc<RefCell<Environment>>`, lo que permite scopes anidados y closures que capturan el entorno de definición.
- Los errores de parseo se acumulan y se muestran al usuario en el REPL.
- `build.rs` ejecuta `git rev-parse --short HEAD` para incluir el hash del commit en el banner.

## Limitaciones actuales

- Solo enteros de 64 bits; no hay soporte para floats.
- Los identificadores admiten letras y `_`, pero no dígitos (`addTen` sí, `add10` no).
- `len()` funciona únicamente con strings.
- Los arrays existen como literales, pero todavía no hay indexación (`arr[0]`) ni builtins como `first`, `last`, `rest` o `push`.
- No hay hash maps, loops, comentarios, módulos ni asignación fuera de `let`.
- El REPL evalúa línea por línea; no se admiten definiciones de funciones multilínea.
- No existe modo de ejecución por archivo; por ahora el único modo es el REPL.
- El manejo de EOF (`Ctrl+D`) está pendiente; usar `Ctrl+C` para salir.

## Roadmap

- Indexación de arrays y builtins de colecciones.
- Hash maps.
- Soporte de floats.
- Modo script para ejecutar archivos `.monkey`.
- Recuperación de errores más robusta en el parser.
- Manejo correcto de EOF en el REPL.
- Integración continua con GitHub Actions.

## Créditos

- Inspirado en *Writing an Interpreter in Go* de Thorsten Ball y en el lenguaje Monkey.
- Implementación en Rust sin dependencias externas.
