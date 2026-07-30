# Markdown Architectural Decision Records (MADR) Guide

This reference details the structure, fields, and conventions of MADR v3.0 format for architectural decision capture.

## Purpose

MADR provides a lean, readable, and structured Markdown format for documenting software architecture decisions directly within a git repository.

## Key Fields & Sections

### Title

The title should state the decision as a short, noun-phrase or active verb phrase:

- `0001-record-architecture-decisions.md`
- `0002-use-fastapi-for-rest-endpoints.md`

### Status

The status line records the current lifecycle state of the decision:

- **Proposed**: Under review, pending team alignment.
- **Accepted**: Decision agreed upon and currently active.
- **Rejected**: Evaluated but decided against.
- **Deprecated**: Previously accepted but no longer applicable.
- **Superseded by [ADR-YYYY](YYYY-title.md)**: Replaced by a newer decision record.

### Technical Story

Brief reference to the underlying ticket, issue, or request driving the decision (e.g., `Issue #104` or `Feature/auth-v2`).

### Context & Problem Statement

Describes the forces, requirements, constraints, and operational environment forcing a choice. Explain *why* a decision must be made now.

### Decision Drivers

Key factors influencing the outcome (e.g., performance, operational cost, compliance, developer velocity).

### Considered Options

List all candidate solutions evaluated during the decision-making process.

### Decision Outcome

Explicitly states the chosen option and key rationale, followed by immediate consequences (pros and cons).

### Pros and Cons of Options

Detailed comparison matrix for each considered option to record counterfactual trade-offs for future maintainers.
