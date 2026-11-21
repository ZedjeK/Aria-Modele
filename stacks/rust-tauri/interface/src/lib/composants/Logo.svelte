<!--
  Le logo de la famille Aria : plaque à coins coupés, A en tubes néon (jambe gauche cyan, jambe droite bleu roi,
  sommet blanc décalé), et l'élément orange propre à l'appli au pied du A. Même dessin qu'Aria Studio et Aria Cockpit.
  Élément orange de cette appli : {{ELEMENT_ORANGE}}. Le losange ci-dessous le remplace en attendant son dessin ; le
  même dessin va dans appli\icons\icon.ico.
-->
<script lang="ts">
  interface Props {
    taille?: number;
  }

  let { taille = 44 }: Props = $props();
  const a = '30,130 70,30 90,30 130,130 110,130 80,54 50,130';
</script>

<svg width={taille} height={taille} viewBox="0 0 160 160" aria-hidden="true">
  <defs>
    <clipPath id="logo-bas-gauche"><rect x="0" y="72" width="80" height="90" /></clipPath>
    <clipPath id="logo-bas-droit"><rect x="80" y="72" width="80" height="90" /></clipPath>
    <clipPath id="logo-haut"><rect x="0" y="0" width="160" height="72" /></clipPath>
    <filter id="logo-halo" x="-40%" y="-40%" width="180%" height="180%">
      <feGaussianBlur stdDeviation="3.2" result="flou" />
      <feMerge><feMergeNode in="flou" /><feMergeNode in="flou" /><feMergeNode in="SourceGraphic" /></feMerge>
    </filter>
  </defs>
  <polygon points="24,0 136,0 160,24 160,136 136,160 24,160 0,136 0,24" fill="#030812" />
  <polygon
    points="26,7 134,7 153,26 153,134 134,153 26,153 7,134 7,26"
    fill="none"
    stroke="#4d7cff"
    stroke-width="1.8"
    opacity="0.65"
  />
  <g filter="url(#logo-halo)" fill="none" stroke-width="5" stroke-linejoin="round">
    <polygon points={a} stroke="#7df9ff" clip-path="url(#logo-bas-gauche)" />
    <polygon points={a} stroke="#5c8bff" clip-path="url(#logo-bas-droit)" />
    <polygon points={a} stroke="#ffffff" clip-path="url(#logo-haut)" transform="translate(7 0)" />
    <!-- Élément orange de l'appli : à dessiner. -->
    <polygon points="80,92 94,106 80,120 66,106" fill="#ff8a1f" stroke="none" />
  </g>
</svg>
