import android.os.HwBinder;
import android.os.HwParcel;
import android.os.IHwBinder;
import android.util.Log;
import java.io.FileWriter;
import java.lang.reflect.Constructor;
import java.lang.reflect.Method;

/**
 * MonitorTool — 显示器硬件控制工具
 *
 * 用法:
 *   MonitorTool get <key>                    — 读取 JNI 配置值
 *   MonitorTool set <key> <value> [isUpdate] — 写入 JNI 配置值
 *   MonitorTool getMinMax <key>              — 查询值范围 [min, max]
 *   MonitorTool setColorGains <r> <g> <b>    — 设置 RGB 白平衡增益
 *   MonitorTool dump                         — 批量读取所有画面参数
 *   MonitorTool led <mode> [args...]         — 控制 RGB LED
 *       led off                              — 关闭
 *       led ambient                          — 屏幕同色
 *       led cycle                            — 七彩循环
 *       led lighting [brightness] [colorTemp] — 照明模式
 *       led solid [brightness] [color]       — 纯色模式
 *       led raw <mode> <color>               — 原始模式
 */
public class MonitorTool {
    private static final String TAG = "MonitorTool";
    private static final String RESULT_FILE = "/data/data/mitv.service/cache/.jni_result";

    // MTK TV middleware classpath
    private static final String TV_CLASSPATH =
            "/system/framework/mitvmiddlewareimpl.jar:/system_ext/priv-app/TvServices/TvServices.apk";
    private static final String NATIVE_LIB_PATH = "/vendor/lib64:/system/lib64:/system_ext/lib64";

    // LED HIDL interfaces
    private static final String BSP_IFACE = "vendor.mi.hardware.bspmserver@2.0::IBspMServer";
    private static final String CONFIG_IFACE = "vendor.mi.hardware.bspmserver@2.0::IVendorConfig";

    // ========== Logging & Result ==========

    private static void log(String msg) {
        System.out.println(msg);
        Log.d(TAG, msg);
    }

    private static void writeResult(String value) {
        try {
            FileWriter fw = new FileWriter(RESULT_FILE);
            fw.write(value);
            fw.close();
        } catch (Exception e) {
            Log.e(TAG, "Failed to write result file: " + e.getMessage());
        }
    }

    // ========== MTK JNI Helpers ==========

    private static ClassLoader tvClassLoader() throws Exception {
        return (ClassLoader) Class.forName("dalvik.system.PathClassLoader")
                .getConstructor(String.class, String.class, ClassLoader.class)
                .newInstance(TV_CLASSPATH, NATIVE_LIB_PATH, ClassLoader.getSystemClassLoader());
    }

    private static void cmdGet(ClassLoader cl, String key) throws Exception {
        Class<?> tvNative = cl.loadClass("com.mediatek.twoworlds.tv.TVNative");
        Method get = tvNative.getDeclaredMethod("getConfigValue_native", int.class, String.class);
        get.setAccessible(true);
        int val = (Integer) get.invoke(null, -1, key);
        log("RESULT: GET " + key + " = " + val);
        writeResult(String.valueOf(val));
    }

    private static void cmdSet(ClassLoader cl, String key, int value, int isUpdate) throws Exception {
        Class<?> tvNative = cl.loadClass("com.mediatek.twoworlds.tv.TVNative");
        Method set = tvNative.getDeclaredMethod(
                "setConfigValue_native", int.class, String.class, int.class, int.class);
        set.setAccessible(true);
        int ret = (Integer) set.invoke(null, -1, key, value, isUpdate);
        log("RESULT: SET " + key + " to " + value + " (isUpdate=" + isUpdate + "), return: " + ret);
        writeResult(String.valueOf(ret));
    }

    private static void cmdGetMinMax(ClassLoader cl, String key) throws Exception {
        Class<?> tvNative = cl.loadClass("com.mediatek.twoworlds.tv.TVNative");
        Method getMinMax = tvNative.getDeclaredMethod(
                "getMinMaxConfigValue_native", int.class, String.class);
        getMinMax.setAccessible(true);
        int minVal = (Integer) getMinMax.invoke(null, 0, key);
        int maxVal = (Integer) getMinMax.invoke(null, 1, key);
        String result = minVal + "," + maxVal;
        log("RESULT: RANGE " + key + " = [" + minVal + ", " + maxVal + "]");
        writeResult(result);
    }

    private static void cmdSetColorGains(ClassLoader cl, int red, int green, int blue) throws Exception {
        Class<?> displayClass = cl.loadClass("com.mediatek.twoworlds.factory.MtkTvFApiDisplay");
        Object display = displayClass.getMethod("getInstance").invoke(null);

        Class<?> enumClass = cl.loadClass(
                "com.mediatek.twoworlds.factory.common.MtkTvFApiDisplayTypes$EnumColorTemperature");
        Object userColorTemp = Enum.valueOf((Class<Enum>) enumClass.asSubclass(Enum.class),
                "E_MTK_FAPI_COLOR_TEMP_USER");

        Class<?> dataClass = cl.loadClass(
                "com.mediatek.twoworlds.factory.common.MtkTvFApiDisplayTypes$ColorTempData");
        Constructor<?> ctor = dataClass.getConstructor(
                int.class, int.class, int.class, int.class, int.class, int.class);
        Object data = ctor.newInstance(red, green, blue, 1024, 1024, 1024);

        Method method = displayClass.getMethod("setWbGainOffsetEx", enumClass, dataClass, int.class);
        int ret = (Integer) method.invoke(display, userColorTemp, data, 0);
        log("RESULT: SET_COLOR_GAINS r=" + red + " g=" + green + " b=" + blue + ", return: " + ret);
        writeResult(String.valueOf(ret));
    }

    private static void cmdSetHdrToneMapping(ClassLoader cl, int value, int isUpdate) throws Exception {
        Class<?> tvNative = cl.loadClass("com.mediatek.twoworlds.tv.TVNative");
        Method set = tvNative.getDeclaredMethod(
                "setConfigValue_native", int.class, String.class, int.class, int.class);
        set.setAccessible(true);
        int configRet = (Integer) set.invoke(null, -1, "g_video__vid_hdr_tone_mapping_mode", value, isUpdate);

        Integer hdrAttrRet = null;
        try {
            Class<?> providerClass = cl.loadClass("xiaomi.hardware.mitv.provider.V1_0.IMiTVProvider");
            Object provider = providerClass.getMethod("getService", boolean.class).invoke(null, false);
            if (provider != null) {
                hdrAttrRet = (Integer) providerClass.getMethod("setHdrAttr", int.class, int.class)
                        .invoke(provider, 4, value);
            }
        } catch (Exception e) {
            log("WARN: setHdrAttr failed: " + e.getMessage());
        }
        log("RESULT: SET_HDR_TONE_MAPPING value=" + value + ", configReturn: " + configRet
                + ", hdrAttrReturn: " + (hdrAttrRet == null ? "N/A" : hdrAttrRet));
        writeResult(String.valueOf(configRet));
    }

    private static void cmdDump(ClassLoader cl) throws Exception {
        Class<?> tvNative = cl.loadClass("com.mediatek.twoworlds.tv.TVNative");
        Method get = tvNative.getDeclaredMethod("getConfigValue_native", int.class, String.class);
        get.setAccessible(true);

        StringBuilder sb = new StringBuilder();
        String[] keys = {
                "g_disp__disp_back_light",
                "g_video__brightness",
                "g_video__contrast",
                "g_video__vid_sat",
                "g_video__vid_hue",
                "g_video__vid_shp",
                "g_video__vid_local_dimming",
                "g_video__clr_temp",
                "g_video__clr_gain_r",
                "g_video__clr_gain_g",
                "g_video__clr_gain_b",
                "g_video__vid_mjc_effect",
                "g_video__vid_dejudder"
        };
        for (String k : keys) {
            try {
                int val = (Integer) get.invoke(null, -1, k);
                log(k + ": " + val);
                sb.append(k).append("=").append(val).append("\n");
            } catch (Exception e) {
                log(k + ": ERROR (" + e.getMessage() + ")");
                sb.append(k).append("=ERROR\n");
            }
        }
        writeResult(sb.toString());
    }

    // ========== LED (HwBinder) Helpers ==========

    private static IHwBinder getLedConfigBinder() throws Exception {
        IHwBinder bsp = HwBinder.getService(BSP_IFACE, "default");
        HwParcel req = new HwParcel();
        HwParcel reply = new HwParcel();
        req.writeInterfaceToken(BSP_IFACE);
        try {
            bsp.transact(6, req, reply, 0);
            reply.verifySuccess();
            req.releaseTemporaryStorage();
            int status = reply.readInt32();
            IHwBinder config = reply.readStrongBinder();
            if (status != 0 || config == null) {
                throw new RuntimeException("CreateConfigInterface failed status=" + status);
            }
            return config;
        } finally {
            reply.release();
        }
    }

    private static int setLedColor(int mode, int color) throws Exception {
        IHwBinder config = getLedConfigBinder();
        HwParcel req = new HwParcel();
        HwParcel reply = new HwParcel();
        req.writeInterfaceToken(CONFIG_IFACE);
        req.writeInt32(mode);
        req.writeInt32(color);
        try {
            config.transact(2, req, reply, 0);
            reply.verifySuccess();
            req.releaseTemporaryStorage();
            return reply.readInt32();
        } finally {
            reply.release();
        }
    }

    private static int lightingColor(int illumination, int colorTemp) {
        illumination = clamp(illumination, 0, 14);
        colorTemp = clamp(colorTemp, 0, 2);
        if (colorTemp == 1) return 13 + illumination;   // 4000K
        if (colorTemp == 0) return 28 + illumination;   // 2700K
        return 43 + illumination;                         // 6500K
    }

    private static int solidColor(int illumination, int colorValue) {
        illumination = clamp(illumination, 0, 14);
        colorValue = clamp(colorValue, 0, 4);
        int[] base = {58, 73, 103, 88, 118};  // ice-blue, gold, azure, grass, sunset
        return base[colorValue] + illumination;
    }

    private static int clamp(int v, int lo, int hi) {
        return Math.max(lo, Math.min(hi, v));
    }

    private static int parse(String s, int def) {
        try { return Integer.parseInt(s); } catch (Exception e) { return def; }
    }

    private static void cmdLed(String[] args) throws Exception {
        if (args.length < 2) {
            log("ERROR: led requires a subcommand: off|ambient|cycle|lighting|solid|raw");
            writeResult("ERROR");
            return;
        }
        String sub = args[1];
        int mode, color;
        if ("off".equals(sub)) {
            mode = 0; color = 1;
        } else if ("ambient".equals(sub)) {
            mode = 12; color = 1;
        } else if ("cycle".equals(sub)) {
            mode = 3; color = 1;
        } else if ("lighting".equals(sub)) {
            mode = 11;
            int illum = parse(args.length > 2 ? args[2] : "9", 9);
            int ctemp = parse(args.length > 3 ? args[3] : "1", 1);
            color = lightingColor(illum, ctemp);
        } else if ("solid".equals(sub)) {
            mode = 1;
            int illum = parse(args.length > 2 ? args[2] : "9", 9);
            int cval = parse(args.length > 3 ? args[3] : "0", 0);
            color = solidColor(illum, cval);
        } else if ("raw".equals(sub)) {
            mode = parse(args.length > 2 ? args[2] : "0", 0);
            color = parse(args.length > 3 ? args[3] : "0", 0);
        } else {
            log("ERROR: Unknown led subcommand: " + sub);
            writeResult("ERROR");
            return;
        }
        int ret = setLedColor(mode, color);
        log("SetColorfulLed mode=" + mode + " color=" + color + " ret=" + ret);
        writeResult(String.valueOf(ret));
    }

    // ========== Main ==========

    public static void main(String[] args) throws Exception {
        if (args.length == 0) {
            log("Usage: MonitorTool <get|set|getMinMax|setColorGains|setHdrToneMapping|dump|led> [args]");
            return;
        }

        String cmd = args[0];

        try {
            if ("led".equals(cmd)) {
                cmdLed(args);
            } else {
                // MTK JNI commands
                ClassLoader cl = tvClassLoader();
                String key = args.length >= 2 ? args[1] : null;

                if ("get".equals(cmd)) {
                    if (key == null) { log("ERROR: key required"); writeResult("ERROR"); return; }
                    cmdGet(cl, key);
                } else if ("set".equals(cmd)) {
                    if (key == null || args.length < 3) { log("ERROR: key and value required"); writeResult("ERROR"); return; }
                    int value = Integer.parseInt(args[2]);
                    int isUpdate = args.length >= 4 ? Integer.parseInt(args[3]) : 1;
                    cmdSet(cl, key, value, isUpdate);
                } else if ("getMinMax".equals(cmd)) {
                    if (key == null) { log("ERROR: key required"); writeResult("ERROR"); return; }
                    cmdGetMinMax(cl, key);
                } else if ("setColorGains".equals(cmd)) {
                    if (args.length < 4) { log("ERROR: setColorGains requires r g b"); writeResult("ERROR"); return; }
                    cmdSetColorGains(cl, Integer.parseInt(args[1]), Integer.parseInt(args[2]), Integer.parseInt(args[3]));
                } else if ("setHdrToneMapping".equals(cmd)) {
                    if (args.length < 2) { log("ERROR: setHdrToneMapping requires value [isUpdate]"); writeResult("ERROR"); return; }
                    int hdrValue = Integer.parseInt(args[1]);
                    int hdrIsUpdate = args.length >= 3 ? Integer.parseInt(args[2]) : 3;
                    cmdSetHdrToneMapping(cl, hdrValue, hdrIsUpdate);
                } else if ("dump".equals(cmd)) {
                    cmdDump(cl);
                } else {
                    log("ERROR: Unknown command: " + cmd);
                    writeResult("ERROR: Unknown command " + cmd);
                }
            }
        } catch (Exception e) {
            log("FATAL ERROR: " + e.getMessage());
            writeResult("FATAL: " + e.getMessage());
            e.printStackTrace();
        }

        System.exit(0);
    }
}
