import { useCliLogStream } from '../hooks/useCliLogStream';

/** Renders nothing; it keeps the CLI log stream subscribed while the app is open. */
function CliLogStream() {
    useCliLogStream();
    return null;
}

export default CliLogStream;
