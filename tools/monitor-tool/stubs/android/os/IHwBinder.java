package android.os;

/** Stub for compilation — real implementation on device */
public interface IHwBinder {
    void transact(int code, HwParcel request, HwParcel reply, int flags);
}
