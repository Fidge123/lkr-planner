## Decisions

### Unique name before creating
Planradar copies asynchronously and answers with a job ID that cannot be queried, so the BL-009 client finds the finished copy by its name in the project list.
Creation is refused when an accessible Planradar project already carries the name, so the name identifies the copy and a project left over from an interrupted run is linked instead of duplicated.
- Comparing the list before and after the copy also identifies it but does not recover a leftover project.

### Link not found
A stored link that matches no Planradar project leaves the choice to the planner, who links another project or creates one.
- Creating a new project automatically would duplicate a project whenever the token lost access to it.

### Link before adjusting
The link is stored right after the project exists and before the adjustment form opens, since the app can neither cancel a copy nor delete a project.
- Linking after the form would leave an unlinked project whenever the form is closed.
