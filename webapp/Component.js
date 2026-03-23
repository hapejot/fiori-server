sap.ui.define(["sap/fe/core/AppComponent"], function (AppComponent) {
"use strict";

return AppComponent.extend("products.demo.Component", {
	metadata: {
		manifest: "json"
	},

	init: function () {
		AppComponent.prototype.init.apply(this, arguments);
		this._resolveIntentNavigation();
	},

	/**
	 * Erkennt den FLP-Intent (SemanticObject) und navigiert zur
	 * passenden Entity-Liste, falls kein Route den initialen Hash matcht.
	 */
	_resolveIntentNavigation: function () {
		var oRouter = this.getRouter();
		var that = this;
		var fnBypassed = function () {
			oRouter.detachBypassed(fnBypassed);
			var sEntity = that._getEntityFromIntent();
			if (sEntity) {
				oRouter.navTo(sEntity + "List");
			}
		};
		oRouter.attachBypassed(fnBypassed);
	},

	_getEntityFromIntent: function () {
		try {
			var oURLParsing = sap.ushell.Container.getService("URLParsing");
			var oHash = oURLParsing.parseShellHash(window.location.hash);
			if (oHash && oHash.semanticObject) {
				return oHash.semanticObject;
			}
		} catch (e) { /* nicht im FLP-Kontext */ }
		return null;
	}
});
});
