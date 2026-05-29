# MCP Process Lifecycle Findings

## Context

On Windows, a single Codex user session created multiple copies of the same stdio MCP servers. The visible symptom was sustained high CPU usage and many `node.exe` processes.

Observed duplicate startup window:

```text
22:27:20  app-server initialize   pid=5008
22:28:23  app-server initialize   pid=8980
22:28:50  app-server initialize   pid=14412
```

Each app-server initialized the configured MCP servers again.

## Evidence

Process tree examples:

```text
cmd.exe
  node.exe npx
    cmd.exe package shim
      node.exe real MCP server
```

For one logical MCP, this produced 3-4 OS processes. When Codex created three app-server runtimes, the same MCP stack was multiplied.

`logs_2.sqlite` showed three distinct app-server process UUIDs:

```text
pid:5008   first 22:27:20
pid:8980   first 22:28:23
pid:14412  first 22:28:50
```

No matching shutdown / exit / connection closed records were found for the first two app-server processes before cleanup.

## Finding

The duplication was not caused by an MCP server recursively spawning itself. It was caused by the client/runtime starting multiple app-server instances, each spawning its own stdio MCP children.

The stdio MCP model has no built-in global singleton or process ownership registry. If the client starts multiple runtimes, each runtime can start another copy of the same server.

## Failure Mode

1. Client starts or resumes multiple app-server runtimes.
2. Each runtime spawns all configured stdio MCP servers.
3. Some old runtimes stop receiving traffic but do not cleanly terminate their process trees.
4. `npx` / `cmd` / shim layers make child cleanup unreliable on Windows.
5. CPU and memory usage grow with each duplicate runtime.

## MCP Developer Requirements

- Implement parent-death handling: exit when stdin closes, when the parent PID disappears, or when heartbeat is missed.
- Use a process group / job object on Windows so the whole tree dies together.
- Avoid `npx` for long-running MCP servers; prefer a stable direct executable entrypoint.
- Add a single-instance guard when the server is stateful or expensive.
- Expose a `shutdown` path and log shutdown reason explicitly.
- Include process identity in startup logs: PID, parent PID, server id, transport, cwd, and config hash.
- Treat duplicate startup with the same config as either reuse, reject, or replace-old, not silent coexistence.

## Client Runtime Requirements

- Do not spawn MCP servers per app-server runtime unless isolation is intentional.
- Reuse MCP processes within the same user session when config is identical.
- On app-server replacement, terminate the old runtime and its MCP process tree before starting the next one.
- Record lifecycle events: spawn, initialize, disconnect, shutdown, kill, orphan cleanup.

## Short Conclusion

Stdio MCP is safe only if the client owns process lifecycle rigorously. Without runtime-level deduplication and process-tree cleanup, repeated app-server initialization multiplies MCP servers and leaves orphaned Node.js processes.
