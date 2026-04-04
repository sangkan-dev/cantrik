-- Example Cantrik Lua plugin (Sprint 13). Copy to `.cantrik/plugins/` in your project.
-- Hooks: on_task_start(task_text), after_write(relative_path)

function on_task_start(task)
  if string.find(string.lower(task), "deploy", 1, true) then
    cantrik.warn("Remember: run tests before deploy.")
    cantrik.require_approval("deploy")
  end
end

function after_write(path)
  if string.match(path, "%.rs$") then
    cantrik.suggest("Run: cargo clippy -- -D warnings")
  end
end
