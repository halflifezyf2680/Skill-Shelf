---
name: autocad-llm-drafting
description: Use LLM to generate AutoCAD drawing scripts (AutoLISP / .NET API / SCR) from natural language descriptions. AutoCAD executes the scripts to produce drawings. Use when the user needs to programmatically create or modify AutoCAD drawings, draft floor plans, generate mechanical parts, batch-process DWG files, or automate repetitive CAD tasks.
---

# AutoCAD LLM Drafting

Use this skill to let LLM generate AutoCAD scripts from natural language descriptions, then AutoCAD executes them to produce drawings.

## API Selection Guide

Choose the right API based on the task:

| API | Language | Best For | How to Run |
|-----|----------|----------|------------|
| **AutoLISP** | LISP | Quick scripts, geometric shapes, layer operations, batch text editing | Save as `.lsp`, load with `APPLOAD` or `(load "script")` in AutoCAD command line |
| **SCR (Script)** | Plain text | Linear command sequences, batch plotting, simple repetitions | Save as `.scr`, run with `SCRIPT` command |
| **.NET API** | C# | Complex geometry, custom entities, database operations, event-driven logic | Compile as DLL, `NETLOAD` in AutoCAD |
| **ObjectARX** | C++ | Maximum performance, custom objects, deep integration | Compile as ARX, `ARX` command to load |

**Default to AutoLISP** for most LLM-generated scripts. It's the simplest to generate, execute, and debug.

## Workflow

1. **Understand the drawing intent**: Ask the user what they want to draw. Clarify dimensions, layers, units, and coordinate system if not specified.
2. **Choose API**: Default AutoLISP. Use .NET only if AutoLISP cannot handle the complexity.
3. **Generate the script**: Write the complete script file. Include all geometry, layers, and annotations.
4. **Add error handling**: AutoLISP scripts should handle missing layers, locked layers, and cancel gracefully.
5. **Deliver with instructions**: Provide the script file and clear load/run instructions.

## AutoLISP Reference

### Basic Structure

```lisp
(defun c:MyCommand ( / )
  ;; Your drawing logic here
  (princ "\nDone.")
)
```

`c:` prefix makes it callable from the AutoCAD command line.

### Common Operations

**Draw a line:**
```lisp
(command "_.LINE" "0,0" "100,100" "")
```

**Draw a rectangle:**
```lisp
(command "_.RECTANGLE" "0,0" "200,100")
```

**Draw a circle:**
```lisp
(command "_.CIRCLE" "50,50" 25)
```

**Create and set a layer:**
```lisp
(command "_.LAYER" "N" "Walls" "C" "3" "Walls" "S" "Walls" "")
```

**Add text:**
```lisp
(command "_.TEXT" "10,50" 5 0 "Hello AutoCAD")
```

**Insert a block:**
```lisp
(command "_.INSERT" "blockname" "50,50" 1 1 0)
```

### Safe Pattern

```lisp
(defun c:DrawFloorPlan ( / old_osmode old_cmdecho)
  (setq old_osmode (getvar "OSMODE"))
  (setq old_cmdecho (getvar "CMDECHO"))
  (setvar "OSMODE" 0)
  (setvar "CMDECHO" 0)

  ;; --- Drawing logic here ---

  (setvar "OSMODE" old_osmode)
  (setvar "CMDECHO" old_cmdecho)
  (princ "\nFloor plan drawn.")
)
```

Always save and restore system variables. Turn off `OSMODE` and `CMDECHO` to prevent snap/echo interference.

## .NET API Reference (C#)

Use only when AutoLISP is insufficient (complex transactions, custom entities, event handling).

### Minimal .NET Command

```csharp
using Autodesk.AutoCAD.ApplicationServices;
using Autodesk.AutoCAD.DatabaseServices;
using Autodesk.AutoCAD.Geometry;
using Autodesk.AutoCAD.Runtime;

public class DrawingCommands
{
    [CommandMethod("DrawCircle")]
    public void DrawCircle()
    {
        Document doc = Application.DocumentManager.MdiActiveDocument;
        Database db = doc.Database;

        using (Transaction tr = db.TransactionManager.StartTransaction())
        {
            BlockTable bt = (BlockTable)tr.GetObject(db.BlockTableId, OpenMode.ForRead);
            BlockTableRecord btr = (BlockTableRecord)tr.GetObject(bt[BlockTableRecord.ModelSpace], OpenMode.ForWrite);

            Circle circle = new Circle(new Point3d(50, 50, 0), Vector3d.ZAxis, 25);
            btr.AppendEntity(circle);
            tr.AddNewlyCreatedDBObject(circle, true);

            tr.Commit();
        }
    }
}
```

Compile as DLL, load with `NETLOAD`, run `DrawCircle` command.

## SCR Script Reference

Simplest format — one command per line, plain text.

```scr
_.CIRCLE 50,50 25
_.LINE 0,0 100,100

_.TEXT 10,50 5 0 Hello
```

Save as `.scr`, run with `SCRIPT` command. No variables, no loops. Good for linear batch operations only.

## Generation Rules

- **Always include units**: Specify if drawing uses mm, cm, inches, or meters. AutoCAD is unitless by default.
- **Use absolute coordinates** unless the user specifies relative. Include `(command "_.OSNAP" "OFF")` at the start if appropriate.
- **Layer discipline**: Create named layers with colors before drawing. Group related geometry on the same layer.
- **Close polylines**: Use `(command "_.PLINE" pt1 pt2 pt3 "_C")` with `_C` to close.
- **No hardcoded paths**: If referencing external files (blocks, images), use relative paths or prompt the user.
- **Add comments**: LISP uses `;; comment`, C# uses `// comment`. Explain sections of non-trivial scripts.
- **Test-friendly**: Structure scripts so they can be run multiple times without duplicating geometry. Check if entities exist before creating.

## Anti-Patterns

- Do not generate ObjectARX C++ code unless explicitly requested. It requires the SDK, compiler setup, and is error-prone.
- Do not use `(entmod)` or `(entmake)` when `(command ...)` suffices. Command calls are more readable and debuggable.
- Do not assume AutoCAD version. Stick to commands available in AutoCAD 2020+.
- Do not generate scripts that modify system files, registry, or run shell commands.
