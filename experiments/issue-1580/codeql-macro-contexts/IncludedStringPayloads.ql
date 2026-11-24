/** Preserve actual include_str expansions, including explicitly unavailable ones. */
import rust

// Narrow helper domains before expansion, literal and control-flow joins.
private class IncludedStringCall extends MacroCall {
  IncludedStringCall() { this.fromSource() and this.getPath().getText() = "include_str" }
}

private string literalState(IncludedStringCall call) {
  exists(call.getMacroCallExpansion().(LiteralExpr)) and result = "literal-expansion"
  or
  call.hasMacroCallExpansion() and
  not exists(call.getMacroCallExpansion().(LiteralExpr)) and result = "other-expansion"
  or
  not call.hasMacroCallExpansion() and result = "unexpanded"
}

private string literalText(IncludedStringCall call) {
  result = call.getMacroCallExpansion().(LiteralExpr).getTextValue()
  or
  not exists(call.getMacroCallExpansion().(LiteralExpr)) and result = ""
}

private string controlFlowState(IncludedStringCall call) {
  exists(call.getMacroCallExpansion().getACfgNode()) and result = "expansion-has-cfg-node"
  or
  not exists(call.getMacroCallExpansion().getACfgNode()) and result = "no-expansion-cfg-node"
}

from IncludedStringCall call
select call, call.getFile().getRelativePath(), call.getLocation().getStartLine(),
  call.getLocation().getStartColumn(), call.getLocation().getEndLine(),
  call.getLocation().getEndColumn(), literalState(call), literalText(call), controlFlowState(call)
