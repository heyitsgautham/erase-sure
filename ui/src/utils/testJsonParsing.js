// Test parsing function for CLI output format validation
function testJsonParsing() {
    const sampleOutput = [
        '{"level":"info","message":"Starting device discovery","timestamp":"2025-09-16T16:52:10.173781055+00:00"}',
        '{"level":"info","message":"Found 2 devices","timestamp":"2025-09-16T16:52:10.209419531+00:00"}',
        '[',
        '  {',
        '    "name": "/dev/zram0",',
        '    "model": null,',
        '    "serial": null,',
        '    "capacity_bytes": 4294967296,',
        '    "bus": null,',
        '    "mountpoints": [',
        '      "[SWAP]"',
        '    ],',
        '    "risk_level": "HIGH"',
        '  },',
        '  {',
        '    "name": "/dev/nvme0n1",',
        '    "model": "WD PC SN740 SDDQMQD-512G-1001",',
        '    "serial": "232367403051",',
        '    "capacity_bytes": 512110190592,',
        '    "bus": "NVMe",',
        '    "mountpoints": [',
        '      "/boot",',
        '      "/"',
        '    ],',
        '    "risk_level": "CRITICAL"',
        '  }',
        ']'
    ];

    // Simulate the parseJsonOutput function
    const parseJsonOutput = (lines) => {
        const nonLogLines = [];
        
        for (const line of lines) {
            if (!line.trim()) continue;
            
            try {
                const parsed = JSON.parse(line);
                if (parsed.level && parsed.timestamp) {
                    continue;
                }
                nonLogLines.push(line);
            } catch {
                nonLogLines.push(line);
            }
        }
        
        if (nonLogLines.length === 0) {
            throw new Error('No JSON output found in command output');
        }
        
        const jsonContent = nonLogLines.join('');
        try {
            return JSON.parse(jsonContent);
        } catch (error) {
            for (let i = nonLogLines.length - 1; i >= 0; i--) {
                try {
                    return JSON.parse(nonLogLines[i]);
                } catch {
                    continue;
                }
            }
            throw new Error(`Failed to parse JSON output: ${error}`);
        }
    };

    try {
        const result = parseJsonOutput(sampleOutput);
        console.log('Parsing successful:', result);
        console.log('Device count:', result.length);
        console.log('First device:', result[0]);
        return true;
    } catch (error) {
        console.error('Parsing failed:', error);
        return false;
    }
}

// Export for use in browser console
if (typeof window !== 'undefined') {
    window.testJsonParsing = testJsonParsing;
}

export default testJsonParsing;
