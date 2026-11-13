## Decisions

### Trigger
Reactivation only happens when the planner chooses it in the edit modal.
- Reactivating after every assignment write would reopen projects that were archived on purpose and spend a Planradar request on each write.

### Confirming the result
After the archive request the project is read again and counts as reactivated only when its status is active, because Planradar answers a body with an unknown key with success and changes nothing.
- Trusting the response status alone would report a reactivation that did not happen.

### Already active
A project that is active when the planner asks is not sent to Planradar again.
