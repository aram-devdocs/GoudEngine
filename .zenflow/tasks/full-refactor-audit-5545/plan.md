# Full SDD workflow

## Configuration
- **Artifacts Path**: {@artifacts_path} → `.zenflow/tasks/{task_id}`

---

## Workflow Steps

### [x] Step: Requirements
<!-- chat-id: 46b49ac7-ac11-4828-845e-e7025096003c -->

Create a Product Requirements Document (PRD) based on the feature description.

1. Review existing codebase to understand current architecture and patterns
2. Analyze the feature definition and identify unclear aspects
3. Ask the user for clarifications on aspects that significantly impact scope or user experience
4. Make reasonable decisions for minor details based on context and conventions
5. If user can't clarify, make a decision, state the assumption, and continue

Save the PRD to `{@artifacts_path}/requirements.md`.

### [x] Step: Technical Specification
<!-- chat-id: 0293f184-5dac-4d07-b598-bbfc3219411a -->

Create a technical specification based on the PRD in `{@artifacts_path}/requirements.md`.

1. Review existing codebase architecture and identify reusable components
2. Define the implementation approach

Save to `{@artifacts_path}/spec.md` with:
- Technical context (language, dependencies)
- Implementation approach referencing existing code patterns
- Source code structure changes
- Data model / API / interface changes
- Delivery phases (incremental, testable milestones)
- Verification approach using project lint/test commands

### [x] Step: Audit Technical Specification
<!-- chat-id: a37c0601-90d0-4e53-b4b8-b19eecd4d6f1 -->

we want to use bevy, we want to build our own ecs. you can reverse engineer from bevy but it jhas to be our code and part of our codebase. update the spec codument to account for that. also be verbose, we want this to be like monogame but with the tools youd fine ind a modenr engin. make the scripting easy because the open source code of the game engin e is strong.
 `{@artifacts_path}/spec.md` is the file to audit and update. ensure high level system architere, leave actual code examples and scripts for actual docs.

**Completed Changes (v2.0):**
- Replaced bevy_ecs dependency with custom Bevy-inspired ECS architecture (fully owned code)
- Added comprehensive ECS documentation: entities, components, systems, archetypes, queries, scheduling
- Added high-level system architecture diagrams (6 layers from Platform to Language Bindings)
- Added data flow architecture showing game loop, events, world state
- Documented MonoGame-inspired developer experience features (SpriteBatch, Content Pipeline, Game class)
- Defined multi-language binding strategy (C#, Python, Lua, TypeScript, Go, Rust)
- Removed low-level code examples, focused on architectural concepts and design decisions
- Added custom physics engine (Rapier-inspired, no external dependency)
- Added detailed component categories and built-in systems tables
- Expanded FFI architecture with handle system and batch operations
- Added success metrics and risk mitigation sections
- Added glossary and target file structure

### [ ] Step: Planning

Create a detailed implementation plan based on `{@artifacts_path}/spec.md`.

1. Break down the work into concrete tasks
2. Each task should reference relevant contracts and include verification steps
3. Replace the Implementation step below with the planned tasks

Rule of thumb for step size: each step should represent a coherent unit of work (e.g., implement a component, add an API endpoint, write tests for a module). Avoid steps that are too granular (single function) or too broad (entire feature).

If the feature is trivial and doesn't warrant full specification, update this workflow to remove unnecessary steps and explain the reasoning to the user.

Save to `{@artifacts_path}/plan.md`.

### [ ] Step: Implementation

This step should be replaced with detailed implementation tasks from the Planning step.

If Planning didn't replace this step, execute the tasks in `{@artifacts_path}/plan.md`, updating checkboxes as you go. Run planned tests/lint and record results in plan.md.
