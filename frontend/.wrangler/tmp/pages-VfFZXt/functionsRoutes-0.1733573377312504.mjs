import { onRequest as __geo_js_onRequest } from "C:\\Users\\rubyj\\drip-drop-workspace\\drip-drop\\frontend\\functions\\geo.js"

export const routes = [
    {
      routePath: "/geo",
      mountPath: "/",
      method: "",
      middlewares: [],
      modules: [__geo_js_onRequest],
    },
  ]