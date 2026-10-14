# client-composition: later

What the design names, or the planning found, but this plan does not do.

| Item | Design | When |
|---|---|---|
| Drawings acted on: a drawing bound by its owner, its drawing code reporting `Act::Drawing` with the owner's bytes; this plan adds the act's place only (jig 01.4) | `client/domain/view.md`, 5 and 16 | the first drawing a person acts on |
| The browser's shell, mapping landmarks, region names, shortcuts, menus, disclosures and pictures | `client/domain/view.md`, 11 and 16; `protocol/client.md`, 12 | its own plan, once this plan's tree has landed |
| `ops`'s terminal client's loop over a real terminal (this plan plays it in `ops`'s client world only, jig 10.4) | `examples.md`, 8 and 9.1 | `ops` standalone |
| An application's own child domains, owners that are no connector | `client/domain/root.md`, 15.1 | an application with such a domain |
| An application's streams | `client/domain/contract.md`, 7.4 | an application whose people follow its data as it comes |
| The text shell's words in a language other than English | `client/domain/view.md`, 14 | a second language |
| The field name `connector` of `jig-domain-client`'s `Part`, and `family` in `jig-protocol-client-link`'s exchanges, which carry an owner number (decisions.md, 5) | `client/domain/root.md`, 15; `protocol/client.md`, 11 | a design pass that renames them, if an owner that is no connector names a part |
| The composer inside an agent's activity, where `pages.md` 12.2 lists `jig-composer`; the code builds it in the confirmation's dialog, and this plan names it there (decisions.md, 13) | `client/domain/pages.md`, 4, 5.2 and 12.2 | a design pass on the conversation's page |
| `Conversations { project }` is ignored when the page opens (`crates/jig-domain-client/src/pages/routing.rs:169-172` at the base): the section names its project, the read does not filter by it | `client/domain/pages.md`, 4; `client.md`, 5.1 | when an application shows one project's conversations |
| The project's overview sets the inbox's filter but asks no inbox read (`crates/jig-domain-client/src/pages/project.rs:148-152` at the base), unchanged by this plan | `client/domain/pages.md`, 6.1 | if the overview's needs come late in a story |
