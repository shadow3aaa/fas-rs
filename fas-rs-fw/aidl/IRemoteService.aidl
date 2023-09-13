interface IRemoteService {
    /** Send pkg name & frametime to fas-rs server. */
    boolean sendFrameData(String Pkg, long FrameTimeNanos);
    /** Send pid to server to check cgroup to determine if it is topapp */
    void sendPid(int pid);
}