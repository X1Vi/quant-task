import React, { useState, useEffect } from 'react';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  Cell
} from 'recharts';

function OrderBook() {
  const [messages, setMessages] = useState([]);
  const [isPaused, setIsPaused] = useState(false);
  const [stats, setStats] = useState({ asks: 0, bids: 0, total: 0 });

  // Fetch data every 500ms
  useEffect(() => {
    if (isPaused) return;

    const fetchData = async () => {
      try {
        const response = await fetch('http://localhost:3001/api/messages');
        const data = await response.json();
        setMessages(data);

        // Calculate stats
        const asks = data.filter(m => String.fromCharCode(m.side) === 'A').length;
        const bids = data.filter(m => String.fromCharCode(m.side) === 'B').length;
        setStats({ asks, bids, total: data.length });
      } catch (error) {
        console.error('Failed to fetch:', error);
      }
    };

    fetchData(); // Initial fetch
    const interval = setInterval(fetchData, 500);

    return () => clearInterval(interval);
  }, [isPaused]);

  // Prepare chart data - group by price level
  const chartData = messages.reduce((acc, msg) => {
    const price = (msg.price / 1e9).toFixed(2);
    const side = String.fromCharCode(msg.side);
    
    const existing = acc.find(item => item.price === price);
    if (existing) {
      if (side === 'A') existing.askSize += msg.size;
      else existing.bidSize += msg.size;
    } else {
      acc.push({
        price,
        askSize: side === 'A' ? msg.size : 0,
        bidSize: side === 'B' ? msg.size : 0,
      });
    }
    return acc;
  }, []).sort((a, b) => parseFloat(b.price) - parseFloat(a.price));

  return (
    <div style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <h1 style={styles.title}>📊 Live Order Book</h1>
        <button onClick={() => setIsPaused(!isPaused)} style={styles.button}>
          {isPaused ? '▶️ Resume' : '⏸️ Pause'}
        </button>
      </div>

      {/* Stats */}
      <div style={styles.stats}>
        <div style={styles.statBox}>
          <span style={styles.statLabel}>🟢 Bids:</span>
          <span style={styles.statValue}>{stats.bids}</span>
        </div>
        <div style={styles.statBox}>
          <span style={styles.statLabel}>🔴 Asks:</span>
          <span style={styles.statValue}>{stats.asks}</span>
        </div>
        <div style={styles.statBox}>
          <span style={styles.statLabel}>📦 Total:</span>
          <span style={styles.statValue}>{stats.total}</span>
        </div>
      </div>

      {/* Price Distribution Chart */}
      <div style={styles.chartContainer}>
        <h3 style={styles.subtitle}>Price Level Distribution</h3>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="price" />
            <YAxis />
            <Tooltip />
            <Legend />
            <Bar dataKey="bidSize" fill="#10b981" name="Bid Size" />
            <Bar dataKey="askSize" fill="#ef4444" name="Ask Size" />
          </BarChart>
        </ResponsiveContainer>
      </div>

      {/* Order Book Table */}
      <div style={styles.tableContainer}>
        <table style={styles.table}>
          <thead>
            <tr style={styles.headerRow}>
              <th style={styles.th}>Sequence</th>
              <th style={styles.th}>Price</th>
              <th style={styles.th}>Size</th>
              <th style={styles.th}>Side</th>
              <th style={styles.th}>Action</th>
              <th style={styles.th}>Order ID</th>
            </tr>
          </thead>
          <tbody>
            {messages
              .sort((a, b) => b.sequence - a.sequence)
              .map((msg) => {
                const side = String.fromCharCode(msg.side);
                const action = String.fromCharCode(msg.action);
                const isBid = side === 'B';
                
                return (
                  <tr 
                    key={msg.sequence} 
                    style={{
                      ...styles.row,
                      backgroundColor: isBid ? '#ecfdf5' : '#fef2f2'
                    }}
                  >
                    <td style={styles.td}>{msg.sequence}</td>
                    <td style={{...styles.td, fontWeight: 'bold'}}>
                      ${(msg.price / 1e9).toFixed(2)}
                    </td>
                    <td style={styles.td}>{msg.size}</td>
                    <td style={styles.td}>
                      <span style={{
                        ...styles.badge,
                        backgroundColor: isBid ? '#10b981' : '#ef4444'
                      }}>
                        {isBid ? 'BID' : 'ASK'}
                      </span>
                    </td>
                    <td style={styles.td}>
                      {action === 'A' ? 'Add' : 
                       action === 'M' ? 'Modify' : 
                       action === 'C' ? 'Cancel' : action}
                    </td>
                    <td style={{...styles.td, fontSize: '0.85em'}}>
                      {msg.order_id}
                    </td>
                  </tr>
                );
              })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

const styles = {
  container: {
    padding: '20px',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    maxWidth: '100%',
    margin: '0 auto',
    flex: 1,
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '20px',
  },
  title: {
    margin: 0,
    fontSize: '2em',
  },
  button: {
    padding: '10px 20px',
    fontSize: '16px',
    border: 'none',
    borderRadius: '8px',
    backgroundColor: '#3b82f6',
    color: 'white',
    cursor: 'pointer',
    fontWeight: 'bold',
  },
  stats: {
    display: 'flex',
    gap: '20px',
    marginBottom: '30px',
  },
  statBox: {
    padding: '15px 25px',
    backgroundColor: '#f3f4f6',
    borderRadius: '10px',
    display: 'flex',
    gap: '10px',
    alignItems: 'center',
  },
  statLabel: {
    fontSize: '14px',
    color: '#6b7280',
  },
  statValue: {
    fontSize: '24px',
    fontWeight: 'bold',
    color: '#111827',
  },
  chartContainer: {
    marginBottom: '30px',
    padding: '20px',
    backgroundColor: 'white',
    borderRadius: '10px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
  },
  subtitle: {
    marginTop: 0,
    marginBottom: '15px',
    color: '#374151',
  },
  tableContainer: {
    overflowX: 'auto',
    backgroundColor: 'white',
    borderRadius: '10px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
  },
  table: {
    width: '100%',
    borderCollapse: 'collapse',
  },
  headerRow: {
    backgroundColor: '#f9fafb',
  },
  th: {
    padding: '12px',
    textAlign: 'left',
    borderBottom: '2px solid #e5e7eb',
    fontWeight: '600',
    color: '#374151',
  },
  row: {
    transition: 'background-color 0.2s',
  },
  td: {
    padding: '12px',
    borderBottom: '1px solid #e5e7eb',
  },
  badge: {
    padding: '4px 8px',
    borderRadius: '4px',
    color: 'white',
    fontSize: '0.75em',
    fontWeight: 'bold',
  },
};

export default OrderBook;
