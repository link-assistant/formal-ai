/** Preserve actual include_str expansions, including explicitly unavailable ones. */
import rust

private string literalState(MacroCall call) {
  exists(call.getMacroCallExpansion().(LiteralExpr)) and result = "literal-expansion"
  or
  call.hasMacroCallExpansion() and
  not exists(call.getMacroCallExpansion().(LiteralExpr)) and result = "other-expansion"
  or
  not call.hasMacroCallExpansion() and result = "unexpanded"
}

private string literalText(MacroCall call) {
  result = call.getMacroCallExpansion().(LiteralExpr).getTextValue()
  or
  not exists(call.getMacroCallExpansion().(LiteralExpr)) and result = ""
}

private string controlFlowState(MacroCall call) {
  exists(call.getMacroCallExpansion().getACfgNode()) and result = "expansion-has-cfg-node"
  or
  not exists(call.getMacroCallExpansion().getACfgNode()) and result = "no-expansion-cfg-node"
}

from MacroCall call
where call.fromSource() and call.getPath().getText() = "include_str"
select call, call.getFile().getRelativePath(), call.getLocation().getStartLine(),
  call.getLocation().getStartColumn(), call.getLocation().getEndLine(),
  call.getLocation().getEndColumn(), literalState(call), literalText(call), controlFlowState(call)
