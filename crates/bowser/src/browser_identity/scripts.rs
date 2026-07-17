//! Browser-side identity and diagnostic scripts.

pub(crate) fn init_script() -> &'static str {
    r#"
(() => {
  try {
    Object.defineProperty(navigator, 'webdriver', { get: () => false, configurable: true });
    const cleanAutomationGlobals = () => {
      for (const key of Object.keys(window)) {
        if (key.startsWith('cdc_') || key.startsWith('__webdriver')) {
          try {
            delete window[key];
          } catch (_) {}
        }
      }
    };
    cleanAutomationGlobals();
    setTimeout(cleanAutomationGlobals, 0);
    const originalQuery = window.navigator.permissions?.query;
    if (originalQuery) {
      window.navigator.permissions.query = (parameters) => {
        if (parameters && parameters.name === 'notifications') {
          return Promise.resolve({ state: 'prompt', onchange: null });
        }
        return originalQuery.call(window.navigator.permissions, parameters);
      };
    }
  } catch (_) {}
})();
"#
}

pub(crate) fn runtime_cleanup_script() -> &'static str {
    r#"
(() => {
  try {
    for (const key of Object.getOwnPropertyNames(window)) {
      if (typeof key === 'string' && (key.startsWith('cdc_') || key.includes('__webdriver'))) {
        try {
          delete window[key];
        } catch (_) {}
      }
    }
  } catch (_) {}
})()
"#
}

/// Returns a browser identity diagnostic script.
pub fn diagnostic_script() -> &'static str {
    r#"
(() => {
  const webglIdentity = () => {
    const canvas = document.createElement('canvas');
    const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
    if (!gl) return null;
    return {
      vendor: String(gl.getParameter(37445) || ''),
      renderer: String(gl.getParameter(37446) || ''),
    };
  };
  const pluginIdentity = () => Array.from(navigator.plugins || []).map((plugin) => ({
    name: String(plugin.name || ''),
    filename: String(plugin.filename || ''),
    description: String(plugin.description || ''),
  }));
  const userAgentData = navigator.userAgentData
    ? {
        brands: Array.from(navigator.userAgentData.brands || []).map((brand) => ({
          brand: String(brand.brand || ''),
          version: String(brand.version || ''),
        })),
        mobile: Boolean(navigator.userAgentData.mobile),
        platform: String(navigator.userAgentData.platform || ''),
      }
    : null;
  const chromeRuntime = window.chrome && window.chrome.runtime;
  const notification = navigator.permissions?.query
    ? navigator.permissions.query({ name: 'notifications' }).then((result) => result.state).catch(() => null)
    : Promise.resolve(null);
  return notification.then((notificationPermission) => ({
    userAgent: String(navigator.userAgent || ''),
    userAgentData,
    platform: String(navigator.platform || ''),
    language: navigator.language ? String(navigator.language) : null,
    languages: Array.from(navigator.languages || []).map((language) => String(language)),
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || null,
    viewport: {
      innerWidth: Number(window.innerWidth || 0),
      innerHeight: Number(window.innerHeight || 0),
      outerWidth: Number(window.outerWidth || 0),
      outerHeight: Number(window.outerHeight || 0),
      screenWidth: Number(window.screen?.width || 0),
      screenHeight: Number(window.screen?.height || 0),
      availWidth: Number(window.screen?.availWidth || 0),
      availHeight: Number(window.screen?.availHeight || 0),
      devicePixelRatio: Number(window.devicePixelRatio || 0),
    },
    webgl: webglIdentity(),
    plugins: pluginIdentity(),
    mimeTypes: {
      objectTag: Object.prototype.toString.call(navigator.mimeTypes),
      length: Number(navigator.mimeTypes?.length || 0),
    },
    chrome: {
      present: !!window.chrome,
      runtimePresent: !!chromeRuntime,
      runtimeType: typeof chromeRuntime,
      runtimeKeyCount: chromeRuntime ? Object.keys(chromeRuntime).length : 0,
    },
    notificationPermission,
    webdriverIsTrue: navigator.webdriver === true,
    automationGlobals: Object.keys(window).filter((key) =>
      key.startsWith('cdc_') || key.includes('__webdriver')
    ),
  }));
})()
"#
}
