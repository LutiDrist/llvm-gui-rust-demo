simple-llvm — README

Коротко и без воды: это минимальный учебный компилятор / интерпретатор на Rust + простая GUI-обёртка на Go.
Проект умеет: лексить → парсить → строить AST → интерпретировать (интерпретатор) → генерировать LLVM IR (codegen via inkwell). GUI запускает core и показывает вывод в браузере.

Ниже — как всё устроено, как собрать и запустить на Ubuntu 24.04 (и как решать типичные ошибки).

Структура проекта (короче)
simple-llvm/
├─ Cargo.toml  (workspace)
├─ core/        (Rust core: lexer, parser, ast, interpreter, codegen, main)
│  ├─ Cargo.toml
│  └─ src/
│     ├─ main.rs
│     ├─ lexer.rs
│     ├─ parser.rs
│     ├─ ast.rs
│     ├─ interpreter.rs
│     └─ codegen.rs
├─ gui/         (Go web GUI)
│  └─ main.go
└─ README.md

Что делает каждая часть (очень просто)

core/ — основа на Rust.

lexer.rs — разбивает текст программы на токены (let, if, цифры, +, == и т.д.).

parser.rs — строит AST (дерево синтаксиса) с приоритетами (* / выше + -, сравнения ниже).

ast.rs — типы AST (Expr, Stmt, Function).

interpreter.rs — выполняет AST прямо (пока всё в памяти): let/assign/if/while/арифметика/сравнения.

codegen.rs — конвертирует AST → LLVM IR через inkwell (печатает IR).

main.rs — демонстрация: лексер → парсер → интерпретатор → генерация IR. Также поддерживает запуск с аргументом — core <path-to-src-file>.

gui/ — Go HTTP-сервер и страница с редактором.

Сервер при нажатии кнопки /run сохраняет введённый код во временный файл и вызывает собранный core бинарь, возвращает stdout/stderr клиенту.

Что нужно установить (Ubuntu 24.04)

Запусти (пример):

sudo apt update
sudo apt install -y build-essential curl git pkg-config cmake \
    llvm-15 clang-15 libclang-dev libpolly-15-dev zlib1g-dev libzstd-dev \
    golang-go


Rust / Cargo:

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# затем перезапусти shell или source ~/.cargo/env


LLVM (если не хватает libs)
Если урывками не хватает библиотек для llvm-sys/inkwell, удобно использовать официальный скрипт:

wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 15


После установки LLVM 15 пропиши переменные окружения (пример для bash/zsh):

export LLVM_CONFIG=$(which llvm-config-15 || which llvm-config)
export LLVM_SYS_150_PREFIX=$($LLVM_CONFIG --prefix)
export LLVM_SYS_150_STATIC=0
# добавить в ~/.profile / ~/.bashrc чтобы сохранялось


Если при линковке вылезают ошибки cannot find -lz или -lzstd — установи zlib1g-dev libzstd-dev (см. apt install выше).

Сборка и запуск

Сборка Rust core:

cd core
cargo build
# или для релиза:
cargo build --release


После этого бинарь будет:

debug: core/target/debug/core

release: core/target/release/core

Если в workspace несколько бинарей: cargo run --bin core — запуск конкретного бина.

Запуск core вручную (демо):

# запуск с встроенной demo-программой (без аргументов)
cargo run --bin core

# или с файлом-исходником
cargo run --bin core -- /path/to/myprog.slang
# (в main.rs код читает файл, если передан путь)


Запуск GUI (Go):

cd gui
go run main.go
# затем открыть http://localhost:8080


GUI шлёт введённый код в Go-сервер, сервер создаёт временный файл и запускает ../core/target/debug/core <tmpfile>.
Важно: GUI ожидает, что core уже собран и находится по пути ../core/target/debug/core относительно gui папки. Если у тебя другая структура — поправь путь в gui/main.go.

Ожидаемый вывод

При запуске demo (core):

вы увидите tokens [...] — список токенов, которые сгенерил лексер.

затем AST: Function { ... } — дерево.

=== Interpreter === — лог исполнения интерпретатора (let x = 0, x = 1, ...).

=== LLVM IR (generated) === — сгенерированный LLVM IR (печатается в stderr/вывод).

Если запускаешь через GUI, то весь stdout/stderr будет показан на странице в <pre>.

Частые ошибки и как их лечить

cargo run не знает, какой бинарь запускать
— используй cargo run --bin core или укажи default-run в Cargo.toml.

Ошибки llvm/inkwell/llvm-sys при сборке (linker, Polly, missing z)

Установи LLVM dev-пакеты (см. выше).

Установи zlib1g-dev libzstd-dev libclang-dev libpolly-15-dev.

Экспортируй LLVM_CONFIG и LLVM_SYS_150_PREFIX как выше.

Если проблема с Polly, запусти ./llvm.sh 15 и/или установи соответствующие libpolly-15-dev.

Если видишь cannot find -lz → sudo apt install zlib1g-dev.

После установки переменных окружения закрой/открой терминал.

Go: imported and not used
— Go не позволяет неиспользуемые импорты. Убери лишние или _ "pkg".

Интерпретатор паника (panic!)
— читаем стектрейс, идём в соответствующий файл (core/src/interpreter.rs) и ищем panic!("..."). Часто — заглушка, нужно реализовать случай (например while или assign).

GUI ничего не делает при нажатии кнопок

Убедись, что GUI запущен: go run main.go → http://localhost:8080.

Убедись, что core собран и лежит по пути, указанному в main.go (по умолчанию ../core/target/debug/core).

Открой DevTools (Network) — посмотреть POST /run и ответ сервера. Там будет stdout/stderr.

В логах GUI (терминал, где запущен go run) может быть ошибка — смотри её.

Примеры кода (toy-language)

Пример программы, которая используется в demo / GUI:

fn main() {
    let x = 0;
    while (x < 5) {
        x = x + 1;
    }
    if (x == 5) {
        42;
    } else {
        0;
    }
}


let x = expr; — объявление и инициализация

x = expr; — присваивание

expr; — выражение (печатается в интерпретаторе)

while (cond) { ... }, if (cond) { ... } else { ... }

арифметика + - * / и сравнения == != < <= > >=

Что дальше (план развития)

Добавить поддержку функций с параметрами и вызовов (AST + codegen sigs + caller/callee).

JIT: выполнить LLVM IR в памяти (ExecutionEngine via inkwell) и вернуть i32 результат main() (чтобы GUI мог показать результат, а не только печать).

Отладчик: реализовать step() в интерпретаторе и связать кнопки GUI для пошагового исполнения.

IR → оптимизации (constant folding / dead code elimination).

GUI: подсветка, визуализация CFG (Graphviz) и окно переменных.

Как вносить правки и где смотреть

Код интерпретатора: core/src/interpreter.rs

Код генерации IR: core/src/codegen.rs

Парсер/лексер: core/src/parser.rs, core/src/lexer.rs

GUI: gui/main.go + HTML шаблон внутри

Если что-то ломается — сначала запускай cargo build в core/ и запускай бинарь вручную, смотри stdout/stderr. Это быстрее, чем дебажить через GUI.

Советы по отладке

Для ошибок компиляции Rust: cargo build -p core --verbose.

Для линковочных проблем с LLVM: llvm-config-15 --libs --system-libs посмотри, что возвращает.

В Go открой DevTools (Network/Console) — посмотри ответ сервера.

Запускай core из командной строки: core path/to/tmpfile — ты увидишь полные логи.

Пример рабочего сценария (быстрый)

Собрать core:

cd core
cargo build


Запустить GUI:

cd ../gui
go run main.go
# открыть http://localhost:8080


Вставить код в textarea → нажать Run (interp + IR) → смотри вывод.

Лицензия / Контрибуция

Этот проект — учебный. Код можно модифицировать. Если хочешь — открывай PR/пиши в issues (если вести репозиторий на GitHub).
