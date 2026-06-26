package android.os;

/** Stub for compilation — real implementation on device */
public class HwBinder implements IHwBinder {
    public static IHwBinder getService(String iface, String name) { return null; }
    @Override public void transact(int code, HwParcel request, HwParcel reply, int flags) {}
}
