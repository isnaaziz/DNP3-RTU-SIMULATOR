import { connect } from "./api.js";
import { initConfigIo } from "./config-io.js";
import { initPointDialog } from "./dialogs/point-dialog.js";
import { initSettingsDialog } from "./dialogs/settings-dialog.js";
import { installInteractions } from "./interactions.js";
import { installPointerGuard } from "./render/index.js";
import { initToolbar } from "./toolbar.js";

installPointerGuard();
installInteractions();
initPointDialog();
initSettingsDialog();
initToolbar();
initConfigIo();
connect();
