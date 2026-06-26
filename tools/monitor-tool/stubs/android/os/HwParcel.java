package android.os;

/** Stub for compilation — real implementation on device */
public class HwParcel {
    public void writeInterfaceToken(String s) {}
    public void writeInt32(int v) {}
    public int readInt32() { return 0; }
    public IHwBinder readStrongBinder() { return null; }
    public void verifySuccess() {}
    public void releaseTemporaryStorage() {}
    public void release() {}
}
