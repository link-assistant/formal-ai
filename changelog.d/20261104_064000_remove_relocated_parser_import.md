Remove the obsolete parent-module import after extracting the structural link
parser policy test, so native test targets compile under warnings-as-errors.
Keep the child module's used import and every policy assertion unchanged.
