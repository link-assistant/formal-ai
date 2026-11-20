/** Observe source macro AST nodes without changing the security query suite. */
import rust

private string expansionState(MacroCall call) {
  call.hasMacroCallExpansion() and result = "expanded"
  or
  not call.hasMacroCallExpansion() and result = "unexpanded"
}

private string enclosingState(MacroCall call) {
  call.isInMacroExpansion() and result = "inside-expansion"
  or
  not call.isInMacroExpansion() and result = "outside-expansion"
}

from MacroCall call
where call.fromSource()
select call, call.getFile().getRelativePath(), call.getLocation().getStartLine(),
  call.getLocation().getStartColumn(), call.getLocation().getEndLine(),
  call.getLocation().getEndColumn(), expansionState(call), enclosingState(call)
