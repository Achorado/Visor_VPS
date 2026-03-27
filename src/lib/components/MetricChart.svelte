<script lang="ts">
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  import type { EChartsOption } from 'echarts';
  import { save } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/api';

  interface Props {
    title?: string;
    color?: string;
    unit?: string;
    data?: Array<[number, number]>; // [timestamp_ms, value]
    max?: number;
    threshold?: { warn: number; danger: number };
    height?: string;
    loading?: boolean;
  }

  let {
    title = '',
    color = '#6366f1',
    unit = '%',
    data = [],
    max = 100,
    threshold,
    height = '200px',
    loading = false,
  }: Props = $props();

  let container = $state<HTMLDivElement>();
  let chart: echarts.ECharts | null = null;

  function getColor(latest: number): string {
    if (!threshold) return color;
    if (latest >= threshold.danger) return '#ef4444';
    if (latest >= threshold.warn) return '#f59e0b';
    return color;
  }

  function getBaseOption(): EChartsOption {
    return {
      animation: true,
      animationDuration: 400,
      animationEasing: 'cubicOut',
      grid: { top: 35, right: 8, bottom: 24, left: 8, containLabel: true },
      toolbox: {
        right: 10,
        top: 0,
        feature: {
          dataZoom: { yAxisIndex: 'none', title: { zoom: 'Zoom', back: 'Reset Zoom' } },
          myDownload: {
            show: true,
            title: 'Save Image',
            icon: 'path://M4.7,22.9L29.3,45.5L54.7,23.4M4.6,43.6L4.6,58L53.8,58L53.8,43.6M29.2,45.1L29.2,0',
            onclick: async function() {
              if (!chart) return;
              const url = chart.getDataURL({ type: 'png', backgroundColor: '#0f172a', pixelRatio: 2 });
              try {
                const filePath = await save({
                  filters: [{ name: 'Image', extensions: ['png'] }],
                  defaultPath: `${title || 'chart'}-${Date.now()}.png`
                });
                if (filePath) {
                  await api.system.saveFile(filePath, url);
                }
              } catch (e) {
                console.error("Failed to save image", e);
              }
            }
          }
        },
        iconStyle: { borderColor: '#64748b' },
        emphasis: { iconStyle: { borderColor: '#f8fafc' } }
      },
      tooltip: {
        trigger: 'axis',
        backgroundColor: 'rgba(15,23,42,0.95)',
        borderColor: 'rgba(255,255,255,0.08)',
        borderWidth: 1,
        textStyle: { color: '#f1f5f9', fontSize: 12, fontFamily: 'Inter' },
        formatter: (params: unknown) => {
          const p = (params as Array<{ value: [number, number] }>)[0];
          if (!p) return '';
          const date = new Date(p.value[0]);
          const time = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
          const val = p.value[1];
          const latestColor = getColor(val);
          return `<div style="font-size:11px;color:#94a3b8">${time}</div>
                  <div style="font-weight:600;color:${latestColor}">${title ? title + ': ' : ''}${val.toFixed(1)}${unit}</div>`;
        },
        axisPointer: { type: 'cross', crossStyle: { color: 'rgba(255,255,255,0.15)' } },
      },
      xAxis: {
        type: 'time',
        axisLine: { lineStyle: { color: 'rgba(255,255,255,0.06)' } },
        axisTick: { show: false },
        axisLabel: {
          color: '#475569',
          fontSize: 10,
          formatter: (val: number) => {
            const d = new Date(val);
            return `${d.getHours().toString().padStart(2,'0')}:${d.getMinutes().toString().padStart(2,'0')}`;
          }
        },
        splitLine: { show: false },
      },
      dataZoom: [
        { type: 'inside', xAxisIndex: 0, filterMode: 'filter' } // keep inside scroll
      ],
      yAxis: {
        type: 'value',
        min: 0,
        max,
        axisLine: { show: false },
        axisTick: { show: false },
        axisLabel: { color: '#475569', fontSize: 10, formatter: (v: number) => `${v}${unit}` },
        splitLine: { lineStyle: { color: 'rgba(255,255,255,0.04)' } },
        ...(threshold ? {
          markLine: {
            silent: true,
            data: [
              { yAxis: threshold.warn, lineStyle: { color: '#f59e0b', type: 'dashed', width: 1 } },
              { yAxis: threshold.danger, lineStyle: { color: '#ef4444', type: 'dashed', width: 1 } },
            ]
          }
        } : {})
      },
      series: [{
        type: 'line',
        data: [],
        smooth: 0.4,
        symbol: 'none',
        lineStyle: { width: 2 },
        areaStyle: { },
        emphasis: { disabled: true }
      }]
    };
  }

  function getDynamicOption(d: Array<[number, number]>): EChartsOption {
    const latest = d.length > 0 ? d[d.length - 1][1] : 0;
    const c = getColor(latest);
    return {
      series: [{
        data: d,
        lineStyle: { color: c },
        areaStyle: {
          color: {
            type: 'linear',
            x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [
              { offset: 0, color: `${c}55` },
              { offset: 1, color: `${c}05` },
            ]
          }
        }
      }]
    };
  }

  onMount(() => {
    chart = echarts.init(container!, 'dark');
    chart.setOption(getBaseOption());
    chart.setOption(getDynamicOption(data));
    const ro = new ResizeObserver(() => chart?.resize());
    ro.observe(container!);
    return () => { ro.disconnect(); chart?.dispose(); };
  });

  $effect(() => {
    chart?.setOption(getDynamicOption(data), { notMerge: false, lazyUpdate: false });
  });
</script>

<div class="metric-chart-wrap" style:height>
  {#if loading}
    <div class="skeleton" style="width:100%;height:100%"></div>
  {:else}
    {#if title}
      <div class="chart-title">{title}</div>
    {/if}
    <div bind:this={container} style="width:100%;height:100%"></div>
  {/if}
</div>

<style>
  .metric-chart-wrap {
    position: relative;
    width: 100%;
  }
  .chart-title {
    position: absolute;
    top: 0;
    left: 8px;
    font-size: 0.7rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 600;
    z-index: 1;
  }
</style>
