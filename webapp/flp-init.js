/**
 * @fileOverview Local FLP bootstrap — replaces the SAP UShell sandbox bootstrap.
 *
 * Loads application definitions from /config/apps.json, transforms them into
 * UShell-compatible tile and navigation target configuration, and boots the
 * Unified Shell in local (CDM) mode.
 *
 * Uses the official xx-bootTask entry point so that UI5 core settings
 * (theme, language, etc.) injected by the server into the HTML are respected.
 */
(function () {
    "use strict";

    // ── helpers ──────────────────────────────────────────────────────

    /**
     * Deep-merge oConfigToMerge into oMutatedBaseConfig (mutates base).
     * Taken from the SAP UShell bootstrap — behaviour-identical.
     */
    function mergeConfig(oMutatedBaseConfig, oConfigToMerge, bClone) {
        var oActual = bClone
            ? JSON.parse(JSON.stringify(oConfigToMerge))
            : oConfigToMerge;

        if (typeof oConfigToMerge !== "object") {
            return;
        }

        Object.keys(oActual).forEach(function (sKey) {
            if (
                Object.prototype.toString.call(oMutatedBaseConfig[sKey]) ===
                    "[object Object]" &&
                Object.prototype.toString.call(oActual[sKey]) ===
                    "[object Object]"
            ) {
                mergeConfig(oMutatedBaseConfig[sKey], oActual[sKey], false);
                return;
            }
            oMutatedBaseConfig[sKey] = oActual[sKey];
        });
    }

    // ── tile / application transform ────────────────────────────────

    function createTile(oApp, iSuffix, sKey) {
        var sTitle = oApp.title || sKey;
        return {
            id: "flp_tile_" + iSuffix,
            title: sTitle,
            size: "1x1",
            tileType: "sap.ushell.ui.tile.StaticTile",
            properties: {
                chipId: "flp_chip_" + iSuffix,
                title: sTitle,
                info: oApp.description || "",
                targetURL: "#" + sKey
            }
        };
    }

    /**
     * Transform the flat `applications` map into the structures that the
     * LaunchPage adapter (tiles / groups) and the NavTargetResolution /
     * ClientSideTargetResolution adapters expect.
     */
    function adjustApplicationConfiguration(oUshellConfig, commonUtils) {
        var oApplicationConfig = {};
        var aKeys = [];
        var sKey;

        if (
            oUshellConfig.applications &&
            typeof oUshellConfig.applications === "object"
        ) {
            for (sKey in oUshellConfig.applications) {
                if (
                    oUshellConfig.applications.hasOwnProperty(sKey) &&
                    sKey !== ""
                ) {
                    aKeys.push(sKey);
                }
            }
        }

        if (!aKeys.length) {
            return oUshellConfig;
        }

        // ── LaunchPage adapter: groups + tiles ──────────────────────
        var oLaunchPageCfg = JSON.parse(
            JSON.stringify(
                getNestedValue(
                    oUshellConfig,
                    "services.LaunchPage.adapter.config"
                ) || {}
            )
        );
        setNestedValue(
            oApplicationConfig,
            "services.LaunchPage.adapter.config",
            oLaunchPageCfg
        );

        if (!oLaunchPageCfg.groups) {
            oLaunchPageCfg.groups = [];
        }

        var oGroup = {
            id: "flp_default_group",
            title: "Applications",
            tiles: []
        };
        oLaunchPageCfg.groups.unshift(oGroup);

        aKeys.forEach(function (sAppKey, idx) {
            oGroup.tiles.push(
                createTile(oUshellConfig.applications[sAppKey], idx, sAppKey)
            );
        });

        // ── NavTargetResolution adapter ─────────────────────────────
        var oNavApps = {};
        setNestedValue(
            oApplicationConfig,
            "services.NavTargetResolution.adapter.config.applications",
            oNavApps
        );
        mergeConfig(oNavApps, oUshellConfig.applications, true);
        delete oUshellConfig.applications;

        // ── service migration (V2 → CDM) ───────────────────────────
        if (commonUtils && typeof commonUtils.getV2ServiceMigrationConfig === "function") {
            var oMigration =
                commonUtils.getV2ServiceMigrationConfig(oApplicationConfig);
            mergeConfig(oUshellConfig, oMigration, true);
        }
        mergeConfig(oUshellConfig, oApplicationConfig, true);

        return oUshellConfig;
    }

    // ── simple nested-value helpers (no ObjectPath dependency) ──────

    function getNestedValue(obj, path) {
        return path.split(".").reduce(function (cur, key) {
            return cur && cur[key];
        }, obj);
    }

    function setNestedValue(obj, path, value) {
        var parts = path.split(".");
        var last = parts.pop();
        var cur = parts.reduce(function (o, key) {
            if (!o[key] || typeof o[key] !== "object") {
                o[key] = {};
            }
            return o[key];
        }, obj);
        cur[last] = value;
    }

    // ── config loading ──────────────────────────────────────────────

    function loadJson(sUrl) {
        var oResult = null;
        var xhr = new XMLHttpRequest();
        xhr.open("GET", sUrl, false); // synchronous – matches SAP pattern
        xhr.setRequestHeader("Accept", "application/json");
        xhr.send();
        if (xhr.status === 200) {
            try {
                oResult = JSON.parse(xhr.responseText);
            } catch (e) {
                /* ignore parse errors */
            }
        }
        return oResult;
    }

    /**
     * Read /config/apps.json and transform each entry into a UShell
     * `applications` record that includes component + URL information.
     */
    function loadApplications(oUshellConfig) {
        var oAppsConfig = loadJson("/config/apps.json");
        if (!oAppsConfig || !oAppsConfig.applications) {
            return;
        }

        // Derive component id and url from the ushell-config (set by server
        // from settings.json) or fall back to sensible defaults.
        var sComponentId =
            (oUshellConfig._flpComponent && oUshellConfig._flpComponent.id) ||
            "products.demo";
        var sResourceRoot =
            (oUshellConfig._flpComponent &&
                oUshellConfig._flpComponent.resourceRoot) ||
            "../";

        if (!oUshellConfig.applications) {
            oUshellConfig.applications = {};
        }

        var oApps = oAppsConfig.applications;
        Object.keys(oApps).forEach(function (sKey) {
            var oApp = oApps[sKey];
            oUshellConfig.applications[sKey] = {
                additionalInformation:
                    "SAPUI5.Component=" + sComponentId,
                applicationType: "URL",
                url: oApp.url || sResourceRoot,
                title: oApp.title || sKey,
                description: oApp.description || ""
            };
        });
    }

    // ── init: load dependencies, then return bootstrap function ────

    function init() {
        return new Promise(function (resolve) {
            sap.ui.require(
                [
                    "sap/ushell/bootstrap/common/common.util",
                    "sap/ushell/utils"
                ],
                function (commonUtils, ushellUtils) {
                    function bootstrap(fnCallback) {
                        if (!window["sap-ushell-config"]) {
                            window["sap-ushell-config"] = {};
                        }
                        var oUshellConfig = window["sap-ushell-config"];

                        // Load applications from external config
                        loadApplications(oUshellConfig);

                        // Remove the private helper property before UShell sees it
                        delete oUshellConfig._flpComponent;

                        // Migrate base config from V2 format (must happen first)
                        commonUtils.migrateV2ServiceConfig(oUshellConfig);

                        // Transform applications → tiles + nav targets + CSTR inbounds
                        adjustApplicationConfiguration(oUshellConfig, commonUtils);

                        // Ensure renderer defaults
                        var oRendererConfig =
                            getNestedValue(
                                oUshellConfig,
                                "renderers.fiori2.componentData.config"
                            ) || {};
                        setNestedValue(
                            oUshellConfig,
                            "renderers.fiori2.componentData.config",
                            oRendererConfig
                        );
                        if (!oRendererConfig.rootIntent) {
                            oRendererConfig.rootIntent = "Shell-home";
                        }

                        // Disable default dependency loading (local mode)
                        var oLoaderCfg =
                            getNestedValue(
                                oUshellConfig,
                                "services.Ui5ComponentLoader.config"
                            ) || {};
                        setNestedValue(
                            oUshellConfig,
                            "services.Ui5ComponentLoader.config",
                            oLoaderCfg
                        );
                        if (!oLoaderCfg.hasOwnProperty("loadDefaultDependencies")) {
                            oLoaderCfg.loadDefaultDependencies = false;
                        }

                        // Copy apps from NavTargetResolutionInternal (populated by
                        // migration) to all three resolution services — same reference,
                        // matching the original SAP sandbox pattern.
                        var oApps = JSON.parse(
                            JSON.stringify(
                                getNestedValue(
                                    oUshellConfig,
                                    "services.NavTargetResolutionInternal.adapter.config.applications"
                                ) || {}
                            )
                        );
                        setNestedValue(
                            oUshellConfig,
                            "services.NavTargetResolution.adapter.config.applications",
                            oApps
                        );
                        setNestedValue(
                            oUshellConfig,
                            "services.NavTargetResolutionInternal.adapter.config.applications",
                            oApps
                        );
                        setNestedValue(
                            oUshellConfig,
                            "services.ClientSideTargetResolution.adapter.config.applications",
                            oApps
                        );

                        // Module paths (if any)
                        if (oUshellConfig.modulePaths) {
                            var oModules = {};
                            Object.keys(oUshellConfig.modulePaths).forEach(
                                function (sPath) {
                                    oModules[sPath.replace(/\./g, "/")] =
                                        oUshellConfig.modulePaths[sPath];
                                }
                            );
                            sap.ui.loader.config({ paths: oModules });
                        }

                        // Load Container AFTER all config is set up (side effects!)
                        ushellUtils
                            .requireAsync(["sap/ushell/Container"])
                            .then(function (aModules) {
                                aModules[0].init("local").then(fnCallback);
                            });
                    }

                    resolve(bootstrap);
                }
            );
        });
    }

    // ── entry point: xx-bootTask ────────────────────────────────────

    window["sap-ui-config"] = {
        "xx-bootTask": function (fnCallback) {
            init().then(function (bootstrap) {
                bootstrap(fnCallback);
            });
        }
    };
})();
