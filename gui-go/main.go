package main

import (
	"bytes"
	"encoding/json"
	"net/http"
    _"io"
	"os"
	"os/exec"
	"path/filepath"
	"text/template"
)

var page = template.Must(template.New("page").Parse(`
<!doctype html>
<html>
<head>
  <meta charset="utf-8"/>
  <title>simple-llvm GUI</title>
  <style>
    body { font-family: Arial; margin: 16px; }
    textarea { width: 100%; height: 300px; font-family: monospace; }
    pre { background:#111; color:#eee; padding: 12px; white-space: pre-wrap; max-height: 400px; overflow:auto; }
    .row { display:flex; gap:8px; margin-top:8px; }
  </style>
</head>
<body>
  <h2>simple-llvm — GUI (demo)</h2>
  <form id="fm">
    <textarea id="code">{{.}}</textarea>
    <div class="row">
      <button type="button" onclick="run()">Run (interp + IR)</button>
      <button type="button" onclick="compile()">Compile (IR only)</button>
    </div>
  </form>

  <h3>Output</h3>
  <pre id="out"></pre>

<script>
async function run() {
  const code = document.getElementById("code").value;
  const res = await fetch("/run", {
    method: "POST",
    headers: {"Content-Type":"application/json"},
    body: JSON.stringify({ src: code, mode: "run" })
  });
  const j = await res.json();
  document.getElementById("out").textContent = j.stdout + "\n" + j.stderr;
}

async function compile() {
  const code = document.getElementById("code").value;
  const res = await fetch("/run", {
    method: "POST",
    headers: {"Content-Type":"application/json"},
    body: JSON.stringify({ src: code, mode: "ir" })
  });
  const j = await res.json();
  document.getElementById("out").textContent = j.stdout + "\n" + j.stderr;
}
</script>
</body>
</html>
`))

type runReq struct {
	Src  string `json:"src"`
	Mode string `json:"mode"` // "run" или "ir"
}

type runResp struct {
	Stdout string `json:"stdout"`
	Stderr string `json:"stderr"`
}

func main() {
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		example := `fn main() {
    let x = 0;
    while (x < 3) {
        x = x + 1;
    }
    if (x == 3) { 99; } else { 0; }
}`
		page.Execute(w, example)
	})

	http.HandleFunc("/run", func(w http.ResponseWriter, r *http.Request) {
		var req runReq
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), 400)
			return
		}
		// создаём временный файл с исходником
		tmpdir := os.TempDir()
		tmpfile := filepath.Join(tmpdir, "simple_llvm_src.rs")
		if err := os.WriteFile(tmpfile, []byte(req.Src), 0644); err != nil {
			http.Error(w, err.Error(), 500)
			return
		}

		// путь к бинарю core (предполагается, что ты собрал)
		// отработай, если у тебя бинарь в другом месте — поправь путь
		coreBin := filepath.Join("..", "core", "target", "debug", "core")
		if _, err := os.Stat(coreBin); os.IsNotExist(err) {
			http.Error(w, "core binary not found at "+coreBin+". Build with `cargo build` in core/.", 500)
			return
		}

		// Запускаем core с аргументом — путь до временного файла
		cmd := exec.Command(coreBin, tmpfile)
		var outBuf, errBuf bytes.Buffer
		cmd.Stdout = &outBuf
		cmd.Stderr = &errBuf
		// run
		err := cmd.Run()
		stdout := outBuf.String()
		stderr := errBuf.String()
		if err != nil {
			// дополним stderr
			stderr = stderr + "\nrun error: " + err.Error()
		}

		// Если попросили только IR — можно вернуть только печать после маркера
		resp := runResp{Stdout: stdout, Stderr: stderr}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	})

	addr := ":8080"
	println("GUI server listening on http://localhost" + addr)
	http.ListenAndServe(addr, nil)
}
