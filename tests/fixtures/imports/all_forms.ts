// 1. Side-effect import
import "reflect-metadata";

// 2. Default import
import React from "react";

// 3. Named imports
import { useState, useEffect } from "react";

// 4. Named imports with alias
import { useState as useReactState } from "react";

// 5. Namespace import
import * as ReactAll from "react";

// 6. Type-only import
import type { FC } from "react";

// 7. Combined default + named
import React2, { useMemo, useCallback as useCb } from "react";

// 8. require()
const fs = require("fs");

// 9. Dynamic import
async function loadModule() {
    const mod = await import("./module");
}
